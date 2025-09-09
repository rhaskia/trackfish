use log::info;
use std::fmt::Display;
use std::time::Instant;

#[derive(Clone, PartialEq)]
pub struct Queue {
    pub queue_type: QueueType,
    pub current_track: usize,
    pub listens: Vec<Listen>,
    pub cached_order: Vec<usize>,
}

impl Queue {
    /// Creates a new queue with a type and tracks
    pub fn new(queue_type: QueueType, tracks: Vec<usize>) -> Self {
        Self {
            queue_type,
            current_track: 0,
            listens: Vec::new(),
            cached_order: tracks,
        }
    }

    /// Creates a new queue with a given type and current track
    pub fn new_from_pos(queue_type: QueueType, current_track: usize) -> Self {
        Self {
            queue_type,
            current_track,
            listens: Vec::new(),
            cached_order: Vec::new(),
        }
    }

    /// Creates a new queue with all tracks
    pub fn all() -> Self {
        Queue {
            queue_type: QueueType::AllTracks,
            current_track: 0,
            listens: Vec::new(),
            cached_order: Vec::new(),
        }
    }

    /// Creates a new queue with all tracks, with a starting track
    pub fn all_pos(idx: usize) -> Self {
        Queue {
            queue_type: QueueType::AllTracks,
            current_track: 0,
            listens: Vec::new(),
            cached_order: vec![idx],
        }
    }

    /// Creates a new radio queue
    pub fn radio(idx: usize, name: String) -> Self {
        Queue {
            queue_type: QueueType::Radio(name),
            current_track: 0,
            listens: Vec::new(),
            cached_order: vec![idx],
        }
    }

    /// The currently playing track in a queue
    pub fn current(&self) -> usize {
        *self.cached_order.get(self.current_track).unwrap_or(&0)
    }

    /// The track index at a given index in the queue
    pub fn track(&self, idx: usize) -> usize {
        self.cached_order[idx]
    }

    /// The length of a given queue
    pub fn len(&self) -> usize {
        self.cached_order.len()
    }

    /// Swaps two tracks inside a queue
    /// Used in the queue dragging code
    pub fn swap(&mut self, index_to_move: usize, position: usize) {
        if index_to_move == position {
            return;
        }
        info!("{index_to_move}, {position}");

        let track = self.cached_order[index_to_move];
        let moving_current = index_to_move == self.current_track;
        #[allow(unused_mut)]
        let mut new_pos;

        if position >= self.current_track && index_to_move < self.current_track {
            self.current_track -= 1;
        } else if position <= self.current_track && index_to_move > self.current_track {
            self.current_track += 1;
        }

        if position + 1 >= self.cached_order.len() {
            self.cached_order.remove(index_to_move);
            self.cached_order.push(track);
            new_pos = self.cached_order.len() - 1;
        } else {
            if index_to_move > position {
                self.cached_order.remove(index_to_move);
                self.cached_order.insert(position, track);
                new_pos = position;
            } else {
                self.cached_order.insert(position + 1, track);
                self.cached_order.remove(index_to_move);
                new_pos = position;
            }
        }

        if moving_current {
            self.current_track = new_pos;
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum QueueType {
    AllTracks,
    Radio(String),
    Artist(String),
    Album(String),
    Genre(String),
    Playlist(String, usize),
    Union(Vec<QueueType>),
    Exclusion(Box<QueueType>),
}

impl Display for QueueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllTracks => f.write_str("All Tracks"),
            Self::Radio(name) => write!(f, "{name} Radio"),
            Self::Playlist(name, _) => write!(f, "{name}"),
            Self::Exclusion(excluded) => f.write_str(&format!("Excluding {excluded}")),
            Self::Artist(artist) => f.write_str(artist),
            Self::Album(album) => f.write_str(album),
            Self::Genre(genre) => f.write_str(genre),
            Self::Union(types) => {
                for queue_type in types {
                    queue_type.fmt(f)?;
                }
                Ok(())
            }
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
        Self {
            id,
            start,
            progress,
            percentage,
        }
    }
}
