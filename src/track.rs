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

pub fn load_tracks(directory: &str) -> Vec<Track> {
    let files = get_song_files(directory).unwrap();
    tracing::info!("{files:?}");

    files.into_iter().map(|file| load_track(file)).collect()
}

pub fn load_track(file: String) -> Track {
    let mut tag = Tag::read_from_path(file.clone()).expect(&format!("Track {file} has no id3 tag"));

    let title = tag.title().unwrap_or_default().to_string();
    let artist = tag.artist().unwrap_or_default().to_string();
    let album = tag.album().unwrap_or_default().to_string();
    let genre = tag.genre().unwrap_or_default().replace("\0", ";");
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
