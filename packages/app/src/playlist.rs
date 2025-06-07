use super::utils::strip_unnessecary;
use crate::Track;
use log::info;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Debug, Clone, PartialEq)]
pub struct Playlist {
    pub name: String,
    pub file: String,
    pub track_paths: Vec<String>,
    pub tracks: Vec<usize>,
}

impl Playlist {
    pub fn new(name: String, dir: String) -> Self {
        Playlist {
            name: name.clone(),
            file: dir + &strip_unnessecary(&name) + ".m3u",
            tracks: Vec::new(),
            track_paths: Vec::new(),
        }
    }

    pub fn load(dir: &str, playlist_file: &str, all_tracks: &Vec<Track>) -> Self {
        let path = PathBuf::from(playlist_file);
        let file = playlist_file.to_string();
        let mut name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let mut track_paths = Vec::new();
        let mut tracks = Vec::new();

        for line in std::fs::read_to_string(&path).unwrap().split("\n") {
            let line = line.trim();
            if line.starts_with("#") {
                if line.starts_with("#PLAYLIST:") {
                    name = line[10..].to_string();
                }
            } else {
                track_paths.push(line.to_string());

                let canon_path = PathBuf::from(dir).clone().join(line);
                info!("{canon_path:?}");
                let path = if PathBuf::from(line).exists() {
                    PathBuf::from(line)
                } else if canon_path.exists() {
                    canon_path
                } else {
                    continue;
                };

                if path.is_file() {
                    let idx = all_tracks
                        .iter()
                        .position(|track: &Track| PathBuf::from(&track.file) == path);
                    if let Some(idx) = idx {
                        tracks.push(idx);
                    }
                } else {
                    // TODO file playlists or not idk
                }
            }
        }

        Self {
            name,
            file,
            track_paths,
            tracks,
        }
    }

    pub fn remove(&mut self, track: usize) {
        let position = self.tracks.iter().position(|t| *t == track);

        if let Some(position) = position {
            self.tracks.remove(position);
        }
    }
}

pub fn get_playlist_files(dir: &str) -> Result<Vec<String>, io::Error> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let filename = path.to_str().unwrap().to_string();

        match path.is_file() {
            true => {
                if path_is_playlist(path) {
                    files.push(filename.into());
                }
            }
            false => {
                let mut dir_files = get_playlist_files(&filename)?;
                files.append(&mut dir_files);
            }
        }
    }

    Ok(files)
}

fn path_is_playlist(path: PathBuf) -> bool {
    path.extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        == "m3u"
}
