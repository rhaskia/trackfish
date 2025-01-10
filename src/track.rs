use crate::queue::QueueType;
use std::fs;
use id3::Tag;
use id3::TagLike;
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub file: String,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub genre: Vec<String>,
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
    strip_unnessecary(str1) == strip_unnessecary(str2)
}

pub fn strip_unnessecary(s: &str) -> String {
    s.chars().filter(|c| !(c.is_whitespace() || c.is_ascii_punctuation())).collect::<String>().to_lowercase()
}

pub fn load_tracks(directory: &str) -> Vec<Track> {
    log::info!("{directory}");
    let files = get_song_files(directory).unwrap();
    log::info!("{files:?}");

    files.into_iter().map(|file| load_track(file)).collect()
}

pub fn load_track(file: String) -> Track {
    let mut tag = Tag::read_from_path(file.clone()).expect(&format!("Track {file} has no id3 tag"));

    let title = tag.title().unwrap_or_default().to_string();
    let artist = tag.artist().unwrap_or_default().to_string();
    let album = tag.album().unwrap_or_default().to_string();
    let genre = tag.genre().unwrap_or_default().split('\0').map(|s| s.to_string()).collect();
    let len = tag.duration().unwrap_or(1) as f64;
    let mut year = String::new();
    if let Some(tag_year) = tag.get("Date") {
        year = tag_year.to_string();
        println!("{year}");
    }

    Track { file, title, artist, album, genre, year, len }
}

fn get_song_files(directory: &str) -> Result<Vec<String>, io::Error> {
    let entries = fs::read_dir(directory)?;

    let mp3_files: Vec<String> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("mp3") {
                path.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(mp3_files)
}

#[derive(Debug)]
pub struct TrackInfo {
    pub genres: Vec<usize>,
    pub artist: usize,
    pub bpm: i32,
}

impl TrackInfo {
    pub fn genres_match(&self, other: &TrackInfo) -> f64 {
        let mut matches = 0;
        let total = other.genres.len().min(self.genres.len());
        for genre1 in &self.genres {
            for genre2 in &other.genres {
                if genre1 == genre2 {
                    matches += 1;
                }
            }
        }
        matches as f64 / total as f64
    }
}
