use crate::{track::{Track, TrackInfo, strip_unnessecary, similar}, audio::AudioPlayer, embed::AutoEncoder};
use log::info;
use ndarray::Array1;
use rand::prelude::*;
use std::{
    collections::HashMap,
    time::{Duration, Instant}, fmt::{Display, Pointer},
};
use rand::prelude::SliceRandom;
use rand::distributions::WeightedIndex;

#[derive(PartialEq)]
pub struct QueueManager {
    all_tracks: Vec<Track>,
    track_info: Vec<TrackInfo>,
    pub artists: Vec<String>,
    pub genres: Vec<String>,
    pub listens: Vec<Listen>,
    current_playing: usize,
    current_started: Instant,
    pub progress: f64,
    pub playing: bool,

    pub current_queue: usize,
    pub queues: Vec<Queue>,
    pub queue_tracks: Vec<Vec<usize>>,
    pub player: AudioPlayer,
    pub encoder: AutoEncoder,
}

// Basic functionality
impl QueueManager {
    pub fn new(all_tracks: Vec<Track>) -> Self {
        let mut rng = thread_rng();
        let current_playing = if all_tracks.len() > 0 {
            rng.gen_range(0..all_tracks.len())
        } else { 0 };

        let mut queue = QueueManager {
            all_tracks: all_tracks.clone(),
            current_playing,
            current_started: Instant::now(),
            listens: Vec::new(),
            progress: 0.0,
            playing: true,
            queues: vec![Queue::all()],
            queue_tracks: vec![],
            current_queue: 0,
            track_info: Vec::new(),
            artists: Vec::new(),
            genres: Vec::new(),
            player: AudioPlayer::new(),
            encoder: AutoEncoder::new().unwrap()
        };

        let mut track_info = Vec::new();

        for track in &all_tracks {
            let mut genres = track.genre.iter().map(|genre| queue.genre_index(genre)).collect::<Vec<usize>>();
            genres.sort_by(|a, b| b.cmp(a));
            let artist = queue.artist_index(&track.artist);
            let genre_vec = queue.encoder.genres_to_vec(track.genre.clone());
            let genre_space = queue.encoder.encode(genre_vec);

            track_info.push(TrackInfo { genres, artist, bpm: 100, genre_space });
        }

        queue.track_info = track_info;

        info!("track: {:?}", queue.current_track());
        if let Some(track) = queue.current_track().cloned() {
            queue.player.play_track(&track.file);
        }

        queue
    }

    pub fn genre_index(&mut self, genre: &str) -> usize {
        if let Some(idx) = self.genres.iter().position(|genre2: &String| similar(genre, genre2)) {
            return idx;
        }
        self.genres.push(strip_unnessecary(genre));
        self.genres.len() - 1
    }

    pub fn artist_index(&mut self, artist: &str) -> usize {
        if let Some(idx) = self.artists.iter().position(|artist2: &String| *artist2 == strip_unnessecary(artist)) {
            return idx;
        }
        self.artists.push(strip_unnessecary(artist));
        self.artists.len()
    }

    // what ??
    pub fn next_track(&mut self) -> usize {
        let current_queue = &mut self.queues[self.current_queue];

        current_queue.current_track += 1;

        if let Some(next) = current_queue.cached_order.get(current_queue.current_track) {
            return *next;
        }

        match current_queue.shuffle_mode {
            ShuffleMode::PlaySimilar => self.next_similar(),
            ShuffleMode::Random => 0, // play next queue
            ShuffleMode::None => 0,
        }
    }

    pub fn get_weights(&mut self) -> Array1<f32> {
        let current = &self.track_info[self.current()];
        let mut weights = Array1::from_vec(vec![0.0; self.all_tracks.len()]);
        let mut dists: Vec<(usize, f32)> = self.track_info.iter().map(|track| track.genres_dist(current)).enumerate().collect();
        dists.sort_by(|(_, a), (_, b)| a.total_cmp(b));

        for (dist, (i, _)) in dists.iter().enumerate() {
            if self.all_tracks[*i].genre[0].is_empty() { 
                info!("{:?}", self.all_tracks[*i].genre);
                continue;
            }
            weights[*i] += (1.0 / (1.0 + (dist as f32 / 4.0 - 2.0).exp()));
        }

        for i in 0..self.listens.len() {
            weights[self.listens[i].id] -= (1.0 / ((i as f32 / 2.0 - 1.0).exp() + 1.0)).max(0.0);
        }

        // for i in 0..weights.len() {
        //     if weights[i] < 0.01 {
        //         weights[i] = 0.0;
        //     }
        // }

        weights = weights.clamp(0.0, 10.0);

        weights
    }

    pub fn next_similar(&mut self) -> usize {
        let mut weights = self.get_weights().to_vec();
        info!("{weights:?}");
        let dist = WeightedIndex::new(weights.clone()).unwrap(); 
        let mut rng = thread_rng();

        let next = dist.sample(&mut rng);
        info!("{:?}", weights[next]);
        next
    }

    pub fn skipback(&mut self) {
        if let Some(current_track) = self.current_track() {
            self.listens.push(Listen::new(
                self.current_playing,
                self.current_started,
                current_track.len,
                self.progress,
            ));
        }

        info!("Skipping {} through", self.player.progress_secs());
        if self.player.progress_secs() < 5.0 {
            if self.listens.len() == 0 { return; }

            self.current_playing = self.listens[self.listens.len() - 2].id;
            self.current_started = Instant::now();

            self.player.play_track(&self.all_tracks[self.current_playing].file);
            self.player.skip();
        } else {
            self.player.set_pos(0.0);
            self.current_started = Instant::now();
        }
    }

    pub fn skip(&mut self) {
        if let Some(current_track) = self.current_track() {
            self.listens.push(Listen::new(
                self.current_playing,
                self.current_started,
                current_track.len,
                self.progress,
            ));
        }

        let next_track = self.next_track();

        self.player.play_track(&self.all_tracks[next_track].file);
        self.player.skip();

        self.current_started = Instant::now();
        self.current_playing = next_track;
    }

    // more so fill out the cached order
    pub fn shuffle_queue(&mut self) {
        // allow for override shuffle mode
        match self.get_queue().shuffle_mode {
            ShuffleMode::PlaySimilar => {
                self.mut_queue().cached_order = Vec::new();
            }, // maybe just clear out listens/weights
            ShuffleMode::Random => {
                let current_type = self.get_queue().queue_type.clone();
                let unshuffled = self.get_matching(current_type);      

                self.mut_queue().cached_order = unshuffled;
            },
            ShuffleMode::None => {},
        }
    } 

    pub fn get_matching(&self, queue_type: QueueType) -> Vec<usize> {
        if queue_type == QueueType::AllTracks { return (0..self.all_tracks.len()).collect(); }

        self.all_tracks.iter().enumerate().filter(|(index, track)| track.matches(queue_type.clone())).map(|(index, track)| index).collect()
    }

    pub fn get_queue(&self) -> &Queue {
        &self.queues[self.current_queue]
    } 
    
    pub fn mut_queue(&mut self) -> &mut Queue {
        &mut self.queues[self.current_queue]
    } 
}

// Queue creation
impl QueueManager {
    pub fn add_artist_queue(&mut self, artist: String) {
        self.queues.push(Queue::new(QueueType::Artist(artist), ShuffleMode::None));
        self.current_queue = self.queues.len() - 1;
    }

    pub fn add_album_queue(&mut self, album: String) {
        self.queues.push(Queue::new(QueueType::Album(album), ShuffleMode::None));
        self.current_queue = self.queues.len() - 1;
    }

    pub fn add_current_artist_queue(&mut self) {
        let artist = self.current_track().cloned().unwrap_or_default().artist;
        self.add_artist_queue(artist);
    }

    pub fn add_current_album_queue(&mut self) {
        let album = self.current_track().cloned().unwrap_or_default().album;
        self.add_album_queue(album);
    }
}

// Small functions
impl QueueManager {
    pub fn toggle_playing(&mut self) {
        self.playing = !self.playing;
        self.player.toggle_playing();
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn playing(&self) -> bool {
        self.playing
    }

    pub fn current(&self) -> usize {
        self.current_playing
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.all_tracks.get(self.current_playing)
    }

    pub fn current_track_title(&self) -> Option<&str> {
        Some(&self.current_track()?.title)
    }

    pub fn current_track_album(&self) -> Option<&str> {
        Some(&self.current_track()?.album)
    }

    pub fn current_track_artist(&self) -> Option<&str> {
        Some(&self.current_track()?.artist)
    }

    pub fn current_track_genres(&self) -> Option<&Vec<String>> {
        Some(&self.current_track()?.genre)
    }

    pub fn next_up(&self) -> Option<Track> {
        let current_queue = &self.queues[self.current_queue];
        Some(self.all_tracks.get(*current_queue.cached_order.get(0)?)?.clone())
    }
}

#[derive(Clone, PartialEq)]
pub struct Queue {
    pub queue_type: QueueType,
    pub current_track: usize,
    pub listens: Vec<Listen>,
    pub cached_order: Vec<usize>,
    pub shuffle_mode: ShuffleMode,
}

impl Queue {
    pub fn new(queue_type: QueueType, shuffle_mode: ShuffleMode) -> Self {
        Self {
            queue_type,
            current_track: 0,
            listens: Vec::new(),
            cached_order: Vec::new(),
            shuffle_mode,
        }
    }

    pub fn new_from_pos(queue_type: QueueType, shuffle_mode: ShuffleMode, current_track: usize) -> Self {
        Self {
            queue_type,
            current_track,
            listens: Vec::new(),
            cached_order: Vec::new(),
            shuffle_mode,
        }
    }

    pub fn all() -> Self {
        Queue {
            queue_type: QueueType::AllTracks, 
            current_track: 0,
            listens: Vec::new(),
            cached_order: Vec::new(),
            shuffle_mode: ShuffleMode::PlaySimilar,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum ShuffleMode {
    PlaySimilar,
    Random,
    None,
}

#[derive(PartialEq, Clone)]
pub enum QueueType {
    AllTracks,
    Artist(String),
    Album(String),
    Genre(String),
    Union(Vec<QueueType>),
    Exclusion(Box<QueueType>),
}

impl Display for QueueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllTracks => f.write_str("All Tracks"),
            Self::Exclusion(excluded) => f.write_str(&format!("Excluding {excluded}")),
            Self::Artist(artist) => f.write_str(artist),
            Self::Album(album) => f.write_str(album),
            Self::Genre(genre) => f.write_str(genre),
            Self::Union(types) => {
                for queue_type in types {
                    queue_type.fmt(f)?;
                }
                Ok(())
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Listen {
    id: usize, // make better than an index
    start: Instant,
    progress: f64,
    percentage: f64,
}

impl Listen {
    pub fn new(id: usize, start: Instant, total_len: f64, progress: f64) -> Self {
        let time = start.elapsed();
        let percentage = time.as_secs_f64() / total_len;
        Self { id, start, progress, percentage }
    }
}

pub struct Shuffler {

}
