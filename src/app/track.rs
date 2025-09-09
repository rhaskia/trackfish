use super::queue::QueueType;
use super::utils::similar;
use crate::database::init_db;
use id3::Tag;
use id3::TagLike;
use log::info;
use crate::database::{get_from_cache, save_to_cache};
use ndarray::Array1;
use rodio::Source;
use std::fmt;
use std::fs;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;

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
    /// Does the track match the given queuetype
    pub fn matches(&self, queue_type: QueueType) -> bool {
        match queue_type {
            QueueType::AllTracks => true,
            QueueType::Radio(_) => false,
            QueueType::Artist(target_artist) => self
                .artists
                .iter()
                .any(|artist| similar(artist, &target_artist)),
            QueueType::Album(album) => similar(&album, &self.album),
            QueueType::Genre(_) => todo!(),
            QueueType::Union(_) => todo!(),
            QueueType::Exclusion(_) => todo!(),
            QueueType::Playlist(_, _) => todo!(),
        }
    }

    /// Artists shared between two tracks
    pub fn shared_artists(&self, other: &Self) -> usize {
        self.artists
            .iter()
            .filter(|e| other.artists.contains(e))
            .collect::<Vec<&String>>()
            .len()
    }

    /// Genres shared between two tracks
    pub fn shared_genres(&self, other: &Self) -> usize {
        self.genres
            .iter()
            .filter(|e| other.genres.contains(e))
            .collect::<Vec<&String>>()
            .len()
    }

    /// Does the track have a given genre
    pub fn has_genre(&self, genre: &str) -> bool {
        self.genres.iter().position(|e| similar(e, genre)).is_some()
    }

    /// Does the track have a given artist
    pub fn has_artist(&self, artist: &str) -> bool {
        self.artists
            .iter()
            .position(|e| similar(e, artist))
            .is_some()
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

/// Loads all tracks recursively in a directory
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
                tracks.push(track.clone());
                save_to_cache(&cache, &track)?;
            }
        }
    }

    info!("Track information loaded in {:?}", started.elapsed());

    Ok(tracks)
}

/// Get artists from a track tag
pub fn get_artists(tag: &Tag) -> Option<Vec<String>> {
    if let Some(artists) = tag.artists() {
        if artists.len() != 0 {
            return Some(artists.into_iter().map(|a| a.to_string()).collect());
        }
    }

    for frame in tag.extended_texts() {
        if frame.description == "ARTISTS" {
            return Some(
                frame
                    .value
                    .split(|c| c == 0 as char || c == ';')
                    .map(|artist| artist.to_string())
                    .filter(|artist| !artist.is_empty())
                    .collect(),
            );
        }
    }

    if let Some(artist) = tag.album_artist() {
        return Some(vec![artist.to_string()]);
    }

    None
}

/// MusicBrainz Mood type
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
    /// Number of shared moods between two tracks
    pub fn shared(&self, other: &Self) -> f32 {
        self.to_vec()
            .iter()
            .zip(other.to_vec())
            .map(|(a, b)| if *a == b { 1.0 } else { 0.0 })
            .sum()
    }

    /// Mood to a vec of booleans
    pub fn to_vec(&self) -> Vec<bool> {
        vec![
            self.acoustic,
            self.aggressive,
            self.electronic,
            self.happy,
            self.party,
            self.relaxed,
            self.sad,
        ]
    }

    /// Vec of booleans to a Mood
    pub fn from_vec(vec: Vec<bool>) -> Self {
        if vec.is_empty() {
            return Self::default();
        }
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

// Displays a mood as a comma seperated list of words
impl fmt::Display for Mood {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let displays = vec![
            "Acoustic",
            "Aggressive",
            "Electronic",
            "Happy",
            "Party",
            "Relaxed",
            "Sad",
        ];
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

/// Returns a mood from a track tag
pub fn get_mood(tag: &Tag) -> Option<Mood> {
    for frame in tag.extended_texts() {
        if frame.description == "ab:mood" {
            let mut values = frame
                .value
                .split("\0")
                .map(|e| e.to_string())
                .filter(|e| !e.is_empty());
            let acoustic = values.next().unwrap() == "Acoustic";
            let aggressive = values.next().unwrap() == "Aggressive";
            let electronic = values.next().unwrap() == "Electronic";
            let happy = values.next().unwrap() == "Happy";
            let party = values.next().unwrap() == "Party";
            let relaxed = values.next().unwrap() == "Relaxed";
            let sad = values.next().unwrap() == "Sad";
            return Some(Mood {
                acoustic,
                aggressive,
                electronic,
                happy,
                party,
                relaxed,
                sad,
            });
        }
    }

    None
}

/// Returns the genres from a track tag
pub fn get_genres(tag: &Tag) -> Vec<String> {
    let mut genres: Vec<String> = tag
        .genre()
        .unwrap_or_default()
        .split('\0')
        .map(|s| s.to_string())
        .collect();

    for frame in tag.extended_texts() {
        if frame.description == "ab:genre" {
            let mut ab_genres = frame.value.split("\0").map(|e| e.to_string()).collect();
            genres.append(&mut ab_genres);
        }
    }

    genres.into_iter().filter(|e| !e.is_empty()).collect()
}

/// Gets a specific frame from a track tag given the descriptor
pub fn get_text(tag: &Tag, key: &str) -> Option<String> {
    for frame in tag.extended_texts() {
        if frame.description == key {
            return Some(frame.value.clone());
        }
    }
    return None;
}

/// Loads a track in for any file type
pub fn load_track(file: String) -> anyhow::Result<Track> {
    let filetype = file.split('.').last().unwrap_or("");

    Ok(match filetype {
        "flac" => load_flac_track(file),
        "ogg" => load_ogg_track(file),
        _ => load_id3_track(file),
    }
    .unwrap_or_default())
}

/// Loads in an OGG type track
pub fn load_ogg_track(file: String) -> anyhow::Result<Track> {
    // let f = std::fs::File::open(&file)?;
    // let tag = lewton::inside_ogg::OggStreamReader::new(f)?;
    //
    // let comments = tag.comment_hdr;
    let track = Track {
        file,
        ..Default::default()
    };

    // for (key, value) in &comments.comment_list {
    //     match key.as_str() {
    //         "TRACKNUMBER" => track.trackno = value.parse()?,
    //         "ARTIST" => track.artists = value.split(|c| c == '\0' || c == ';').map(|a| a.to_string()).collect(),
    //         "GENRE" => track.genres = value.split(|c| c == '\0' || c == ';').map(|g| g.to_string()).collect(),
    //         "TITLE" => track.title = value.clone(),
    //         "ALBUM" => track.album = value.clone(),
    //         _ => {}
    //     }
    // }

    Ok(track)
}

/// Loads in an flac track
pub fn load_flac_track(file: String) -> anyhow::Result<Track> {
    // let tag = metaflac::Tag::read_from_path(&file)?;
    let track = Track { file, ..Default::default() };
    
    // for frame in tag.blocks() {
    //     if let VorbisComment(ref comments) = &frame {
    //         for (key, value) in &comments.comments {
    //             match key.as_str() {
    //                 "TRACKNUMBER" => track.trackno = value[0].parse()?,
    //                 "ARTIST" => track.artists = value.clone(),
    //                 "GENRE" => track.genres = value.clone(),
    //                 "TITLE" => track.title = value[0].clone(),
    //                 "ALBUM" => track.album = value[0].clone(),
    //                 _ => {}
    //             }
    //         }
    //     }
    // }

    Ok(track)
}

/// Loads in a ID3 track, which includes mp3, wav, aiff and more
pub fn load_id3_track(file: String) -> anyhow::Result<Track> {
    let tag = Tag::read_from_path(file.clone())?;
    let source = rodio::Decoder::new(BufReader::new(fs::File::open(file.clone())?))?;

    let title = tag.title().unwrap_or_default().to_string();

    let artists = if let Some(artists) = get_artists(&tag) {
        artists.iter().map(|artist| artist.to_string()).collect()
    } else {
        vec![tag.artist().unwrap_or_default().to_string()]
    };

    let mood = get_mood(&tag);

    let album = tag.album().unwrap_or_default().to_string();
    let genres = get_genres(&tag);
    let len = source
        .total_duration()
        .unwrap_or(Duration::ZERO)
        .as_secs_f64();
    let trackno = tag.track().unwrap_or(1) as usize;

    let mut year = String::new();
    if let Some(tag_year) = tag.get("Date") {
        year = tag_year.to_string();
    }

    Ok(Track {
        file,
        title,
        artists,
        album,
        genres,
        year,
        len,
        mood,
        trackno,
    })
}

/// Returns list of song files in a given directory
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

/// Recursively reads a directory
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

/// Returns the track image information from metadata as bytes
pub fn get_track_image(file: &str) -> Option<Vec<u8>> {
    let filetype = file.split(".").last()?;

    match filetype {
        "flac" => {
            // let tag = metaflac::Tag::read_from_path(&file).ok()?;
            //
            // for frame in tag.blocks() {
            //     if let Block::Picture(picture) = frame {
            //         return Some(picture.data.clone());
            //     }
            // }

            None
        }
        "ogg" => {
            // let f = std::fs::File::open(file).ok()?;
            // let tag = lewton::inside_ogg::OggStreamReader::new(f).ok()?;
            //
            // let comments = tag.comment_hdr;
            // let picture = comments.comment_list.iter().find(|(k, v)| k == "METADATA_BLOCK_PICTURE")?;
            //
            // let mut encoded = picture.1.clone();
            // for _ in 0..(encoded.len() % 4) {
            //     encoded.push('=');
            // }
            //
            // let decoded = base64::prelude::BASE64_STANDARD
            //     .decode(encoded.as_bytes()).unwrap();
            // let picture = metaflac::block::Picture::from_bytes(&decoded).ok()?;
            //
            // Some(picture.data)
            Some(Vec::new())
        }
        _ => {
            let tag = id3::Tag::read_from_path(file).ok()?;
            let picture = tag.pictures().next()?;
            Some(picture.data.clone())
        }
    }
}

/// Is a file an audio file?
fn path_is_audio(path: PathBuf) -> bool {
    match path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
    {
        "mp3" | "opus" | "wav" | "flac" | "ogg" | "aiff" => true,
        _ => false,
    }
}

// Track audio features
#[derive(Debug, Clone, PartialEq)]
pub struct TrackInfo {
    pub mfcc: Array1<f32>,
    pub chroma: Array1<f32>,
    pub spectral: Array1<f32>,
    pub energy: f32,
    pub key: i32,
    pub bpm: f32,
    pub zcr: f32,
}

impl TrackInfo {
    /// Averages a set of track audio features
    pub fn average(tracks: Vec<TrackInfo>) -> TrackInfo {
        let count = tracks.len() as f32;
        TrackInfo {
            mfcc: tracks
                .iter()
                .fold(Array1::zeros(13), |a, b| a + b.mfcc.clone())
                / count,
            chroma: tracks
                .iter()
                .fold(Array1::zeros(13), |a, b| a + b.chroma.clone())
                / count,
            spectral: tracks
                .iter()
                .fold(Array1::zeros(13), |a, b| a + b.spectral.clone())
                / count,
            energy: tracks.iter().map(|t| t.energy).sum::<f32>() / count,
            key: tracks.iter().map(|t| t.key).sum::<i32>() / count as i32,
            bpm: tracks.iter().map(|t| t.bpm).sum::<f32>() / count,
            zcr: tracks.iter().map(|t| t.zcr).sum::<f32>() / count,
        }
    }
}

impl Default for TrackInfo {
    fn default() -> Self {
        TrackInfo {
            mfcc: Array1::zeros(13),
            chroma: Array1::zeros(13),
            spectral: Array1::zeros(13),
            energy: 0.0,
            key: 0,
            bpm: 0.0,
            zcr: 0.0,
        }
    }
}
