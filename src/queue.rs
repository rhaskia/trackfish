use crate::track::Track;
use rand::{Rng, thread_rng};
use std::{
    collections::HashMap,
    time::{Duration, Instant}, fmt::{Display, Pointer},
};
use rand::prelude::SliceRandom;

pub struct QueueManager {
    all_tracks: Vec<Track>,
    pub listens: Vec<Listen>,
    current_playing: usize,
    current_started: Instant,
    pub progress: f64,
    pub playing: bool,

    pub current_queue: usize,
    pub queues: Vec<Queue>,
    pub queue_tracks: Vec<Vec<usize>>,
}

// Basic functionality
impl QueueManager {
    pub fn new(all_tracks: Vec<Track>) -> Self {
        QueueManager {
            all_tracks,
            current_playing: 0,
            current_started: Instant::now(),
            listens: Vec::new(),
            progress: 0.0,
            playing: true,
            queues: vec![Queue::all()],
            queue_tracks: vec![],
            current_queue: 0,
        }
    }

    // what ??
    pub fn next_track(&mut self) {
        let current_queue = &self.queues[self.current_queue];

        //let 
    }

    pub fn skip(&mut self) {
        let current_track = &self.all_tracks[self.current_playing];

        self.listens.push(Listen::new(
            self.current_playing,
            self.current_started,
            current_track.len,
            self.progress,
        ));

        let next_track = self.next_track();

        self.current_started = Instant::now();
        self.current_playing = next_track;
    }

    pub fn shuffle_queue(&mut self) {
        let current_queue = &mut self.queues[self.current_queue];

        // allow for override shuffle mode
        match current_queue.shuffle_mode {
            ShuffleMode::PlaySimilar => {}, // maybe just clear out listens/weights
            ShuffleMode::Random => {
                let unshuffled = self.get_matching(current_queue.queue_type);      

                current_queue.cached_order = unshuffled;
            },
        }
    } 

    pub fn get_matching(&self, queue_type: QueueType) -> Vec<usize> {
        if queue_type == QueueType::AllTracks { return (0..self.all_tracks.len()).collect(); }

        self.all_tracks.iter().enumerate().filter(|(track, index)| track.matches(queue_type)).map(|(_, index)| index).collect()
    }
}

// Queue creation
impl QueueManager {
    pub fn add_artist_queue(&mut self, artist: &str) {
        self.queues.push(Queue::new(QueueType::Artist(artist.to_string()), ShuffleMode::None));
        self.current_queue = self.queues.len() - 1;
    }

    pub fn add_album_queue(&mut self, album: &str) {
        self.queues.push(Queue::new(QueueType::Album(album.to_string()), ShuffleMode::None));
        self.current_queue = self.queues.len() - 1;
    }
}

// Small functions
impl QueueManager {
    pub fn toggle_playing(&mut self) {
        self.playing = !self.playing;
    }

    pub fn play(&mut self) {
        self.playing = true
    }

    pub fn playing(&self) -> bool {
        self.playing
    }

    pub fn current(&self) -> usize {
        self.current_playing
    }


    pub fn next_up(&self) -> Track {
        let current_queue = self.queues[self.current_queue];
        self.all_tracks[current_queue[0]].clone()
    }
}

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

pub enum ShuffleMode {
    PlaySimilar,
    Random,
}

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
            Self::Exclusion(excluded) => f.write_fmt("Excluding {}", excluded),
            Self::Artist(artist) => f.write_str(artist),
            Self::Album(album) => f.write_str(album),
            Self::Genre(genre) => f.write_str(genre),
            Self::Union(types) => {
                for queue_type in types {
                    f.fmt(queue_type)?;
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
