use super::queue::QueueType;
use super::utils::similar;
use id3::Tag;
use id3::TagLike;
use log::info;
use ndarray::Array1;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use crate::database::{init_db, get_from_cache, save_to_cache};
use rodio::{Decoder, Source};
use std::io::BufReader;


#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub file: String,
    pub title: String,
    pub album: String,
    pub artists: Vec<String>,
    pub genres: Vec<String>,
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
        self.genres.iter().filter(|e| other.genres.contains(e)).collect::<Vec<&String>>().len()
    }

    pub fn has_genre(&self, genre: &str) -> bool {
        self.genres.iter().position(|e| similar(e, genre)).is_some()
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
            genres: Default::default(),
            year: Default::default(),
            mood: Default::default(),
            trackno: 1,
            len: 100.0,
        }
    }
}

pub fn load_tracks(directory: &str) -> anyhow::Result<Vec<Track>> {
    info!("Loading tracks from {directory}");
    let cache = init_db()?;

    let files = get_song_files(directory)?;
    let started = std::time::SystemTime::now();
    info!("Loaded {} tracks", files.len());
    let mut tracks = Vec::new();

    for file in files {
        match get_from_cache(&cache, &file)? {
            Some(track) => {
                tracks.push(track);
            }
            None => {
                let track = load_track(file)?;
                save_to_cache(&cache, &track)?;
                tracks.push(track);
            }
        }
    }

    info!("Track information loaded in {:?}", started.elapsed());

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

#[derive(PartialEq, Clone, Debug, Default)]
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
        self.to_vec().iter().zip(other.to_vec()).map(|(a, b)| if *a == b { 1.0 } else { 0.0 }).sum()
    }

    pub fn to_vec(&self) -> Vec<bool> {
        vec![self.acoustic, self.aggressive, self.electronic, self.happy, self.party, self.relaxed, self.sad]
    }

    pub fn from_vec(vec: Vec<bool>) -> Self {
        if vec.is_empty() { return Self::default(); }
        Self {
            acoustic: vec[0],
            aggressive: vec[1],
            electronic: vec[2],
            happy: vec[3],
            party: vec[4],
            relaxed: vec[5],
            sad: vec[6],
        }
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

pub fn load_track(file: String) -> anyhow::Result<Track> {
    let tag = Tag::read_from_path(file.clone())?;
    let source = Decoder::new(BufReader::new(fs::File::open(file.clone())?))?;

    let title = tag.title().unwrap_or_default().to_string();

    let artists = if let Some(artists) = get_artists(&tag) {
        artists.iter().map(|artist| artist.to_string()).collect()
    } else {
        vec![tag.artist().unwrap_or_default().to_string()]
    };

    let mood = get_mood(&tag);

    let album = tag.album().unwrap_or_default().to_string();
    let genres = get_genres(&tag);
    let len = source.total_duration().unwrap_or(Duration::ZERO).as_secs_f64();
    let trackno = tag.track().unwrap_or(1) as usize;

    let mut year = String::new();
    if let Some(tag_year) = tag.get("Date") {
        year = tag_year.to_string();
    }

    //let file = PathBuf::from(file).file_name().unwrap().to_str().unwrap().to_string();

    Ok(Track { file, title, artists, album, genres, year, len, mood, trackno })
}

fn get_song_files(directory: &str) -> Result<Vec<String>, io::Error> {
    // Can't seem to load paths with tildes in them
    let expanded = if let Some(home) = dirs::home_dir() {
        directory.replace("~", &home.display().to_string())
    } else { 
        directory.to_string()
    };

    let files = recursive_read_dir(&expanded)?;

    Ok(files)
}

fn recursive_read_dir(dir: &str) -> Result<Vec<String>, io::Error> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let filename = path.to_str().unwrap().to_string();

        match path.is_file() {
            true => {
                if path_is_audio(path) {
                    files.push(filename.into());
                } 
            }
            false => {
                let mut dir_files = recursive_read_dir(&filename)?;
                files.append(&mut dir_files);
            }
        }
    }

    Ok(files)
}

fn path_is_audio(path: PathBuf) -> bool {
    match path.extension().unwrap_or_default().to_str().unwrap_or_default() {
        "mp3" | "opus" | "wav" | "flac" | "ogg" => true,
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackInfo {
    pub genre_space: Array1<f32>,
    pub mfcc: Array1<f32>,
    pub chroma: Array1<f32>,
    pub key: i32,
    pub bpm: i32,
}

impl TrackInfo {
    pub fn genres_dist(&self, other: &TrackInfo) -> f32 {
        let diff = (self.genre_space.clone() - other.genre_space.clone()).pow2();
        diff.sum().sqrt()
    }
}
