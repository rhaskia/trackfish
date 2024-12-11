use crate::queue::QueueType;

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub file: String,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub genre: String,
    pub year: String,
    pub len: f64,
}

impl Track {
    pub fn matches(&self, queue_type: QueueType) -> bool {
        match queue_type {
            QueueType::AllTracks => true,
            QueueType::Artist(artist) => similar(&artist, &self.artist),
            QueueType::Album(album) => similar(&album, &self.album),
            QueueType::Genre(_) => todo!(),
            QueueType::Union(_) => todo!(),
            QueueType::Exclusion(_) => todo!(),
        }
    }
}

pub fn similar(str1: &str, str2: &str) -> bool {
    strip_unnessecary(str1).to_lowercase() == strip_unnessecary(str2).to_lowercase()
}

pub fn strip_unnessecary(s: &str) -> String {
    s.chars().filter(|c| !(c.is_whitespace() || c.is_ascii_punctuation())).collect()
}
