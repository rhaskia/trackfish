use crate::{
    audio::AudioPlayer,
    embed::AutoEncoder,
    track::{Mood, Track, TrackInfo},
    queue::{QueueType, Queue, Listen},
    utils::{similar, title_case, lerp}
};
use log::info;
use ndarray::Array1;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::{
    fmt::Display,
    time::Instant,
};

#[derive(PartialEq)]
pub struct MusicController {
    pub all_tracks: Vec<Track>,
    pub track_info: Vec<TrackInfo>,
    pub artists: Vec<(String, usize)>,
    pub genres: Vec<(String, usize)>,
    pub albums: Vec<(String, usize)>,
    pub listens: Vec<Listen>,
    current_started: Instant,

    pub current_queue: usize,
    pub queues: Vec<Queue>,
    pub player: AudioPlayer,
    pub encoder: AutoEncoder,
    pub radio: RadioSettings,
    pub shuffle: ShuffleSettings,
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

#[derive(Clone, PartialEq)]
pub struct ShuffleSettings {
    pub active: bool,
}

impl ShuffleSettings {
    pub fn new() -> Self {
        Self { active: false }
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }
}

// Basic functionality
impl MusicController {
    pub fn new(all_tracks: Vec<Track>) -> Self {
        let mut rng = thread_rng();
        let current_playing =
            if all_tracks.len() > 0 { rng.gen_range(0..all_tracks.len()) } else { 0 };

        let mut queue = MusicController {
            all_tracks: all_tracks.clone(),
            current_started: Instant::now(),
            listens: Vec::new(),
            queues: vec![Queue::radio(
                current_playing,
                all_tracks.get(current_playing).cloned().unwrap_or_default().title,
            )],
            current_queue: 0,
            track_info: Vec::new(),
            artists: Vec::new(),
            genres: Vec::new(),
            albums: Vec::new(),
            player: AudioPlayer::new(),
            encoder: AutoEncoder::new().unwrap(),
            radio: RadioSettings::new(0.5, 0.7, 0.7),
            shuffle: ShuffleSettings::new(),
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

            if let Some(index) = queue.albums.iter().position(|(a, _)| similar(a, &track.album)) {
                queue.albums[index].1 += 1;
            } else {
                queue.albums.push((track.album.clone(), 1));
            }
        }

        queue.genres.sort();
        queue.albums.sort();
        queue.artists.sort();

        queue.track_info = track_info;

        if let Some(track) = queue.current_track().cloned() {
            queue.player.play_track(&track.file);
            info!("Started track {track:?}");
        }

        queue
    }

    pub fn play_track(&mut self, idx: usize) {
        if let Some(current_track) = self.current_track() {
            self.listens.push(Listen::new(
                self.current_track_idx(),
                self.current_started,
                current_track.len,
                self.player.progress_secs(),
            ));
        }

        self.current_started = Instant::now();

        self.player.play_track(&self.all_tracks[idx].file);
    }

    pub fn get_weights(&mut self) -> Array1<f32> {
        let space = self.track_info[self.current_queue().current()].genre_space.clone();
        let current = self.mut_current_queue().mut_radio_genres();

        if *current == Array1::<f32>::zeros(16) {
            *current = space;
        } else {
            info!("Old Space: {current}");
            *current = lerp(current, &space, 0.35);
            info!("New Space: {current}");
        }

        let _ = current;
        let current = self.current_queue().radio_genres();

        let mut weights = Array1::from_vec(vec![0.0; self.all_tracks.len()]);
        let mut dists: Vec<(usize, f32)> = self
            .track_info
            .iter()
            .map(|track| track.genres_dist_from_vec(&current))
            .enumerate()
            .collect();
        dists.sort_by(|(_, a), (_, b)| a.total_cmp(b));

        for (dist, (i, _)) in dists.iter().enumerate() {
            if self.all_tracks[*i].genre.len() == 0 {
                continue;
            }
            weights[*i] += 1.0 / (1.0 + (dist as f32 * self.radio.temp - 2.0).exp());
        }

        for i in 0..self.all_tracks.len() {
            let current_idx = self.current_queue().current();
            if similar(&self.all_tracks[current_idx].album, &self.all_tracks[i].album) {
                weights *= self.radio.album_penalty;
            }
            if self.all_tracks[current_idx].shared_artists(&self.all_tracks[i]) > 0 {
                weights *= self.radio.artist_penalty;
            }
            if let Some(current_mood) = &self.all_tracks[current_idx].mood {
                if let Some(mood) = &self.all_tracks[i].mood {
                    weights *= 0.8 + mood.shared(current_mood) / 17.5;
                }
            }
        }

        for i in &self.current_queue().cached_order {
            //weights[self.listens[i].id] -= (1.0 / ((i as f32 / 2.0 - 1.0).exp() + 1.0)).max(0.0);
            weights[*i] = 0.0;
        }

        // TODO: negative weighting for genres in songs skipped early (maybe)
        // TODO: genre weighting using a subset of the previous radio songs
        // TODO: mfcc closeness rating
        // TODO: mood weighting

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
            if self.queues[self.current_queue].current_track == 0 {
                return;
            }
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

        match current_queue.queue_type {
            QueueType::Radio(_, _) => {
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

    pub fn get_matching(&self, queue_type: QueueType) -> Vec<usize> {
        if queue_type == QueueType::AllTracks {
            return (0..self.all_tracks.len()).collect();
        }

        self.all_tracks
            .iter()
            .enumerate()
            .filter(|(index, track)| track.matches(queue_type.clone()))
            .map(|(index, track)| index)
            .collect()
    }
}

// Queue creation
impl MusicController {
    pub fn add_artist_queue(&mut self, artist: String) {
        let tracks = self.get_tracks_where(|track| track.artists.contains(&artist));
        self.queues.push(Queue::new(QueueType::Artist(artist), tracks));
        self.current_queue = self.queues.len() - 1;
    }

    pub fn add_album_queue(&mut self, album: String) {
        let tracks = self.get_tracks_where(|track| track.album == album);
        self.queues.push(Queue::new(QueueType::Album(album), tracks));
        self.current_queue = self.queues.len() - 1;
    }

    pub fn add_all_queue(&mut self, track: usize) {
        let mut tracks = (0..self.all_tracks.len()).collect();
        if self.shuffle.active {
            tracks = shuffle_with_first(tracks, track);
        }

        let track_pos = tracks.iter().position(|e| *e == track).unwrap();

        for i in 0..self.queues.len() {
            if self.queues[i].queue_type == QueueType::AllTracks {
                self.queues[i].cached_order = tracks;
                self.queues[i].current_track = track_pos;
                self.current_queue = i;
                self.play_track(track);
                self.play();
                info!("{}, {}", self.queues[i].current(), track);
                return;    
            }
        }

        self.queues.push(Queue::new(QueueType::AllTracks, tracks));
        self.current_queue = self.queues.len() - 1;
        self.queues[self.current_queue].current_track = track_pos;
        self.play_track(track);
        self.play();
    }

    pub fn add_current_album_queue(&mut self) {
        let album = self.current_track().cloned().unwrap_or_default().album;
        self.add_album_queue(album);
    }

    pub fn get_tracks_where<F>(&self, condition: F) -> Vec<usize>
    where
        F: Fn(&Track) -> bool,
    {
        self.all_tracks
            .iter()
            .enumerate()
            .filter(|(_, track)| condition(*track))
            .map(|(idx, _)| idx)
            .collect()
    }
}

pub fn shuffle_with_first(mut tracks: Vec<usize>, start: usize) -> Vec<usize> {
    tracks.remove(start);

    // Probably could use a nicer shuffle method later on
    let mut rng = thread_rng();
    tracks.shuffle(&mut rng);

    tracks.insert(0, start);

    tracks
}

// Small functions
impl MusicController {
    pub fn toggle_playing(&mut self) {
        self.player.toggle_playing();
    }

    pub fn play(&mut self) {
        self.player.play();
    }

    pub fn playing(&self) -> bool {
        self.player.playing()
    }

    pub fn current_track_idx(&self) -> usize {
        self.current_queue().current()
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.all_tracks.get(self.current_queue().current())
    }

    pub fn get_track(&self, idx: usize) -> Option<&Track> {
        self.all_tracks.get(idx)
    }

    pub fn current_track_title(&self) -> Option<&str> {
        Some(&self.current_track()?.title)
    }

    pub fn current_track_mood(&self) -> Option<Mood> {
        Some(self.current_track()?.mood.clone()?)
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

    pub fn get_queue(&self, idx: usize) -> &Queue {
        &self.queues[idx]
    }

    pub fn current_queue(&self) -> &Queue {
        &self.queues[self.current_queue]
    }

    pub fn mut_current_queue(&mut self) -> &mut Queue {
        &mut self.queues[self.current_queue]
    }
}
