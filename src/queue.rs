use std::collections::HashMap;
use crate::Track;
use rand::Rng;

pub struct QueueManager {
    queue: Vec<String>,
    all_tracks: Vec<Track>,
    genre_weights: HashMap<String, f64>,
}

impl QueueManager {
    pub fn new(all_tracks: Vec<Track>) -> Self {
        QueueManager { queue: Vec::new(), all_tracks, genre_weights: HashMap::new() }
    }

    pub fn next_song(&mut self) -> Track {
        // if let Some(track) = self.queue.pop() {
        //     return track;
        // }   , 
        let mut rng = rand::thread_rng();

        self.all_tracks[rng.gen_range(0..self.all_tracks.len())].clone()
    }
}
