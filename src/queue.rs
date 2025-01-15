use crate::{track::{Track, TrackInfo, strip_unnessecary, similar}, audio::AudioPlayer, embed::AutoEncoder};
use log::info;
use ndarray::Array1;
use rand::prelude::*;
use std::{
    time::Instant, fmt::{Display, Pointer},
};
use rand::distributions::WeightedIndex;

#[derive(PartialEq)]
pub struct QueueManager {
    pub all_tracks: Vec<Track>,
    track_info: Vec<TrackInfo>,
    pub artists: Vec<(String, usize)>,
    pub genres: Vec<(String, usize)>,
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
    pub radio: RadioSettings,
}

#[derive(Clone, PartialEq)]
pub struct RadioSettings {
    temp: f32,
    album_penalty: f32,
    artist_penalty: f32,
}

impl RadioSettings {
    pub fn new(temp: f32, album_penalty: f32, artist_penalty: f32) -> Self {
        RadioSettings { temp, album_penalty, artist_penalty }
    }
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
            queues: vec![Queue::all_pos(current_playing)],
            queue_tracks: vec![],
            current_queue: 0,
            track_info: Vec::new(),
            artists: Vec::new(),
            genres: Vec::new(),
            player: AudioPlayer::new(),
            encoder: AutoEncoder::new().unwrap(),
            radio: RadioSettings::new(1.0, 0.5, 0.5),
        };

        let mut track_info = Vec::new();

        for track in &all_tracks {
            let genre_vec = queue.encoder.genres_to_vec(track.genre.clone());
            let genre_space = queue.encoder.encode(genre_vec);

            track_info.push(TrackInfo { genres: Vec::new(), artist: 0, bpm: 100, genre_space });

            for genre in track.genre.clone() {
                if let Some(index) = queue.genres.iter().position(|(g, _)| similar(g, &genre)) {
                    queue.genres[index].1 += 1;
                } else {
                    queue.genres.push((title_case(&genre), 1));
                }
            }

            for artist in track.artists.clone() {
                if let Some(index) = queue.artists.iter().position(|(a, _)| similar(a, &artist)) {
                    queue.artists[index].1 += 1;
                } else {
                    queue.artists.push((artist, 1));
                }
            }
        }

        queue.genres.sort();
        queue.artists.sort();

        queue.track_info = track_info;

        if let Some(track) = queue.current_track().cloned() {
            queue.player.play_track(&track.file);
        }

        queue
    }

    pub fn play_track(&mut self, idx: usize) {
        if let Some(current_track) = self.current_track() {
            self.listens.push(Listen::new(
                self.current_playing,
                self.current_started,
                current_track.len,
                self.progress,
            ));
        }

        self.current_playing = idx;
        self.current_started = Instant::now();

        self.player.play_track(&self.all_tracks[self.current_playing].file);
        //self.player.skip();
    }

    pub fn get_weights(&mut self) -> Array1<f32> {
        let current = &self.track_info[self.current()];
        let mut weights = Array1::from_vec(vec![0.0; self.all_tracks.len()]);
        let mut dists: Vec<(usize, f32)> = self.track_info.iter().map(|track| track.genres_dist(current)).enumerate().collect();
        dists.sort_by(|(_, a), (_, b)| a.total_cmp(b));

        for (dist, (i, _)) in dists.iter().enumerate() {
            if self.all_tracks[*i].genre.len() == 0 { 
                continue;
            }
            weights[*i] += 1.0 / (1.0 + (dist as f32 * self.radio.temp - 2.0).exp());
        }

        for i in 0..self.all_tracks.len() {
            if similar(&self.all_tracks[self.current()].album, &self.all_tracks[i].album) {
                weights *= self.radio.album_penalty;
                info!("Tracks shared album");
            }
            if self.all_tracks[self.current()].shared_artists(&self.all_tracks[i]) > 0 {
                weights *= self.radio.artist_penalty;
                info!("Tracks shared artist");
            }
        }

        for i in &self.current_queue().cached_order {
            //weights[self.listens[i].id] -= (1.0 / ((i as f32 / 2.0 - 1.0).exp() + 1.0)).max(0.0);
            weights[*i] = 0.0;
        }

        // TODO: negative weighting based on recent artist
        // TODO: negative weighting for recent albums
        // TODO: negative weighting for genres in songs skipped early (maybe)
        // TODO: genre weighting using a subset of the previous radio songs

        weights = weights.clamp(0.0, 10.0);

        weights
    }

    pub fn next_similar(&mut self) -> usize {
        let weights = self.get_weights().to_vec();
        let dist = WeightedIndex::new(weights.clone()).unwrap(); 
        let mut rng = thread_rng();

        let next = dist.sample(&mut rng);
        next
    }

    pub fn skipback(&mut self) {
        info!("Skipping {} through", self.player.progress_secs());
        if self.player.progress_secs() < 5.0 {
            if self.queues[self.current_queue].current_track == 0 { return; }
            let last = self.queues[self.current_queue].current_track - 1;
            self.queues[self.current_queue].current_track = last;
        }

        self.play_track(self.queues[self.current_queue].current());
    }

    pub fn skip(&mut self) {
        let current_queue = &mut self.queues[self.current_queue];

        current_queue.current_track += 1;

        if let Some(next) = current_queue.cached_order.get(current_queue.current_track).cloned() {
            self.play_track(next);
            return;
        }

        match current_queue.shuffle_mode {
            ShuffleMode::PlaySimilar => {
                let next = self.next_similar();
                self.queues[self.current_queue].cached_order.push(next);
                self.play_track(next);
            }
            _ => {
                if self.queues.len() > self.current_queue + 1 {
                    self.current_queue += 1;     
                    // TODO: shuffle next queue if needed
                    self.play_track(self.current_queue().track(0))
                }
            }
        }
    }

    pub fn set_queue_and_track(&mut self, queue: usize, track: usize) {
        self.current_queue = queue;
        self.queues[queue].current_track = track;
        self.play_track(self.queues[queue].cached_order[track]);
    }

    // more so fill out the cached order
    pub fn shuffle_queue(&mut self) {
        // allow for override shuffle mode
        match self.current_queue().shuffle_mode {
            ShuffleMode::PlaySimilar => {
                self.mut_queue().cached_order = Vec::new();
            }, // maybe just clear out listens/weights
            ShuffleMode::Random => {
                let current_type = self.current_queue().queue_type.clone();
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

    pub fn get_queue(&self, idx: usize) -> &Queue {
        &self.queues[idx]
    }

    pub fn current_queue(&self) -> &Queue {
        &self.queues[self.current_queue]
    } 
    
    pub fn mut_queue(&mut self) -> &mut Queue {
        &mut self.queues[self.current_queue]
    } 
}

// Queue creation
impl QueueManager {
    pub fn add_artist_queue(&mut self, artist: String) {
        let tracks = self.get_tracks_where(|track| track.artists.contains(&artist));
        self.queues.push(Queue::new(QueueType::Artist(artist), ShuffleMode::None, tracks));
        self.current_queue = self.queues.len() - 1;
    }

    pub fn add_album_queue(&mut self, album: String) {
        let tracks = self.get_tracks_where(|track| track.album == album);
        self.queues.push(Queue::new(QueueType::Album(album), ShuffleMode::None, tracks));
        self.current_queue = self.queues.len() - 1;
    }

    pub fn add_all_queue(&mut self, track: usize) {
        self.queues.push(Queue::all_pos(track));
        self.current_queue = self.queues.len() - 1;
    }

    pub fn add_current_album_queue(&mut self) {
        let album = self.current_track().cloned().unwrap_or_default().album;
        self.add_album_queue(album);
    }

    pub fn get_tracks_where<F>(&self, condition: F) -> Vec<usize> 
    where F: Fn(&Track) -> bool {
        self.all_tracks.iter().enumerate().filter(|(_, track)| condition(*track)).map(|(idx, _)| idx).collect() 
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

    pub fn get_track(&self, idx: usize) -> Option<&Track> {
        self.all_tracks.get(idx)
    }

    pub fn current_track_title(&self) -> Option<&str> {
        Some(&self.current_track()?.title)
    }

    pub fn current_track_album(&self) -> Option<&str> {
        Some(&self.current_track()?.album)
    }

    pub fn current_track_artist(&self) -> Option<&Vec<String>> {
        Some(&self.current_track()?.artists)
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
    pub fn new(queue_type: QueueType, shuffle_mode: ShuffleMode, tracks: Vec<usize>) -> Self {
        Self {
            queue_type,
            current_track: 0,
            listens: Vec::new(),
            cached_order: tracks,
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

    // Radio QueueType
    pub fn all_pos(idx: usize) -> Self {
        Queue {
            queue_type: QueueType::AllTracks, 
            current_track: 0,
            listens: Vec::new(),
            cached_order: vec![idx],
            shuffle_mode: ShuffleMode::PlaySimilar,
        }
    }

    pub fn current(&self) -> usize {
        self.cached_order[self.current_track]
    }

    pub fn track(&self, idx: usize) -> usize {
        self.cached_order[idx]
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

pub fn title_case(s: &str) -> String {
    let mut result = String::new();
    let mut last_whitespace = true;

    for c in s.chars() {
        if last_whitespace {
            result.push_str(&c.to_uppercase().collect::<String>());
        } else {
            result.push(c);
        }
        last_whitespace = c.is_whitespace();
    }

    result
}
