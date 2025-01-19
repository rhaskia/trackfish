use crate::queue::QueueType;
use crate::utils::similar;
use id3::Tag;
use id3::TagLike;
use id3::frame::ExtendedText;
use log::info;
use ndarray::Array1;
use std::fmt;
use std::fs;
use std::io;

use super::embed::AutoEncoder;

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub file: String,
    pub title: String,
    pub album: String,
    pub artists: Vec<String>,
    pub genre: Vec<String>,
    pub mood: Option<Mood>,
    pub trackno: usize,
    pub year: String,
    pub len: f64,
}

impl Track {
    pub fn matches(&self, queue_type: QueueType) -> bool {
        match queue_type {
            QueueType::AllTracks => true,
            QueueType::Radio(_, _) => false,
            QueueType::Artist(target_artist) => {
                self.artists.iter().any(|artist| similar(artist, &target_artist))
            }
            QueueType::Album(album) => similar(&album, &self.album),
            QueueType::Genre(_) => todo!(),
            QueueType::Union(_) => todo!(),
            QueueType::Exclusion(_) => todo!(),
        }
    }

    pub fn shared_artists(&self, other: &Self) -> usize {
        self.artists.iter().filter(|e| other.artists.contains(e)).collect::<Vec<&String>>().len()
    }

    pub fn shared_genres(&self, other: &Self) -> usize {
        self.genre.iter().filter(|e| other.genre.contains(e)).collect::<Vec<&String>>().len()
    }

    pub fn has_genre(&self, genre: &str) -> bool {
        self.genre.iter().position(|e| similar(e, genre)).is_some()
    }

    pub fn has_artist(&self, artist: &str) -> bool {
        self.artists.iter().position(|e| similar(e, artist)).is_some()
    }
}

impl Default for Track {
    fn default() -> Self {
        Self {
            file: String::new(),
            title: String::from("No Track Selected"),
            album: Default::default(),
            artists: Default::default(),
            genre: Default::default(),
            year: Default::default(),
            mood: Default::default(),
            trackno: 1,
            len: 100.0,
        }
    }
}

pub fn load_tracks(directory: &str) -> anyhow::Result<Vec<Track>> {
    let encoder = AutoEncoder::new()?;
    let files = get_song_files(directory)?;
    info!("Loaded {} tracks", files.len());
    let mut tracks = Vec::new();

    for file in files {
        tracks.push(load_track(file, &encoder)?);
    }

    Ok(tracks)
}

pub fn get_artists(tag: &Tag) -> Option<Vec<String>> {
    for frame in tag.extended_texts() {
        if frame.description == "ARTISTS" {
            return Some(
                frame
                    .value
                    .split("\0")
                    .map(|artist| artist.to_string())
                    .filter(|artist| !artist.is_empty())
                    .collect(),
            );
        }
    }

    None
}

#[derive(PartialEq, Clone, Debug)]
pub struct Mood {
    acoustic: bool,
    aggressive: bool,
    electronic: bool,
    happy: bool,
    party: bool,
    relaxed: bool,
    sad: bool,
}

impl Mood {
    pub fn shared(&self, other: &Self) -> f32 {
        let mut result = 0.0;
        if self.acoustic == other.acoustic { result += 1.0; }
        if self.aggressive == other.aggressive { result += 1.0; }
        if self.electronic == other.electronic { result += 1.0; }
        if self.happy == other.happy { result += 1.0; }
        if self.party == other.party { result += 1.0; }
        if self.relaxed == other.relaxed { result += 1.0; }
        if self.sad == other.sad { result += 1.0; }
        result
    }
}

impl fmt::Display for Mood {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let displays =
            vec!["Acoustic", "Aggressive", "Electronic", "Happy", "Party", "Relaxed", "Sad"];
        let keep = vec![
            self.acoustic,
            self.aggressive,
            self.electronic,
            self.happy,
            self.party,
            self.relaxed,
            self.sad,
        ];
        write!(
            f,
            "{}",
            displays
                .into_iter()
                .zip(keep)
                .filter(|(_, keep)| *keep)
                .map(|(txt, _)| txt)
                .collect::<Vec<&str>>()
                .join(", ")
        )
    }
}

pub fn get_mood(tag: &Tag) -> Option<Mood> {
    for frame in tag.extended_texts() {
        if frame.description == "ab:mood" {
            let mut values =
                frame.value.split("\0").map(|e| e.to_string()).filter(|e| !e.is_empty());
            let acoustic = values.next().unwrap() == "Acoustic";
            let aggressive = values.next().unwrap() == "Aggressive";
            let electronic = values.next().unwrap() == "Electronic";
            let happy = values.next().unwrap() == "Happy";
            let party = values.next().unwrap() == "Party";
            let relaxed = values.next().unwrap() == "Relaxed";
            let sad = values.next().unwrap() == "Sad";
            return Some(Mood { acoustic, aggressive, electronic, happy, party, relaxed, sad });
        }
    }

    None
}

pub fn get_genres(tag: &Tag) -> Vec<String> {
    let mut genres: Vec<String> =
        tag.genre().unwrap_or_default().split('\0').map(|s| s.to_string()).collect();

    for frame in tag.extended_texts() {
        if frame.description == "ab:genre" {
            let mut ab_genres = frame.value.split("\0").map(|e| e.to_string()).collect();
            genres.append(&mut ab_genres);
        }
    }

    genres.into_iter().filter(|e| !e.is_empty()).collect()
}

pub fn get_text(tag: &Tag, key: &str) -> Option<String> {
    for frame in tag.extended_texts() {
        if frame.description == key {
            return Some(frame.value.clone());
        }
    }
    return None;
}

pub fn load_track(file: String, encoder: &AutoEncoder) -> anyhow::Result<Track> {
    let mut tag = Tag::read_from_path(file.clone())?;

    let title = tag.title().unwrap_or_default().to_string();

    let artists = if let Some(artists) = get_artists(&tag) {
        artists.iter().map(|artist| artist.to_string()).collect()
    } else {
        vec![tag.artist().unwrap_or_default().to_string()]
    };

    let mood = get_mood(&tag);

    let album = tag.album().unwrap_or_default().to_string();
    let genre = get_genres(&tag);
    let len = tag.duration().unwrap_or(1) as f64;
    let trackno = tag.track().unwrap_or(1) as usize;

    let mut year = String::new();
    if let Some(tag_year) = tag.get("Date") {
        year = tag_year.to_string();
    }

    Ok(Track { file, title, artists, album, genre, year, len, mood, trackno })
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

#[derive(Debug, PartialEq)]
pub struct TrackInfo {
    pub genres: Vec<usize>,
    pub genre_space: Array1<f32>,
    pub artist: usize,
    pub bpm: i32,
}

impl TrackInfo {
    pub fn genres_dist(&self, other: &TrackInfo) -> f32 {
        let diff = (self.genre_space.clone() - other.genre_space.clone()).pow2();
        diff.sum().sqrt()
    }

    pub fn genres_dist_from_vec(&self, other: &Array1<f32>) -> f32 {
        let diff = (self.genre_space.clone() - other.clone()).pow2();
        diff.sum().sqrt()
    }
}
