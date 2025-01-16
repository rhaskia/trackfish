use ndarray::Array1;
use std::time::Instant;
use std::fmt::Display;

#[derive(Clone, PartialEq)]
pub struct Queue {
    pub queue_type: QueueType,
    pub current_track: usize,
    pub listens: Vec<Listen>,
    pub cached_order: Vec<usize>,
}

impl Queue {
    pub fn new(queue_type: QueueType, tracks: Vec<usize>) -> Self {
        Self {
            queue_type,
            current_track: 0,
            listens: Vec::new(),
            cached_order: tracks,
        }
    }

    pub fn new_from_pos(
        queue_type: QueueType,
        current_track: usize,
    ) -> Self {
        Self {
            queue_type,
            current_track,
            listens: Vec::new(),
            cached_order: Vec::new(),
        }
    }

    pub fn all() -> Self {
        Queue {
            queue_type: QueueType::AllTracks,
            current_track: 0,
            listens: Vec::new(),
            cached_order: Vec::new(),
        }
    }

    pub fn all_pos(idx: usize) -> Self {
        Queue {
            queue_type: QueueType::AllTracks,
            current_track: 0,
            listens: Vec::new(),
            cached_order: vec![idx],
        }
    }

    pub fn radio(idx: usize, name: String) -> Self {
        Queue {
            queue_type: QueueType::Radio(name, Array1::<f32>::zeros(16)),
            current_track: 0,
            listens: Vec::new(),
            cached_order: vec![idx],
        }
    }

    pub fn current(&self) -> usize {
        self.cached_order[self.current_track]
    }

    pub fn track(&self, idx: usize) -> usize {
        self.cached_order[idx]
    }

    pub fn radio_genres(&self) -> Array1<f32> {
        match &self.queue_type {
            QueueType::Radio(_, genres) => genres.clone(),
            _ => panic!("Queue not of type Radio"),
        }
    }

    pub fn mut_radio_genres(&mut self) -> &mut Array1<f32> {
        match &mut self.queue_type {
            QueueType::Radio(_, ref mut genres) => genres,
            _ => panic!("Queue not of type Radio"),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum QueueType {
    AllTracks,
    Radio(String, Array1<f32>),
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
            Self::Radio(name, _) => write!(f, "{name} Radio"),
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
        Self { id, start, progress, percentage }
    }
}

