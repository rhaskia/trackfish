use std::{path::PathBuf, str::FromStr};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Settings {
    pub volume: f32,
    pub directory: String,
    pub radio_temp: f32,
    pub radio_album_penalty: f32,
    pub radio_artist_penalty: f32,
    pub shuffle: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let directory = if cfg!(target_os = "android") {
            "/storage/emulated/0/Music".to_string()
        } else { dirs::audio_dir().unwrap().display().to_string() };
        
        Self { 
            volume: 1.0,
            directory,
            radio_temp: 0.5,
            radio_album_penalty: 0.7,
            radio_artist_penalty: 0.7,
            shuffle: false
        }
    }
}

impl Settings {
    pub fn dir() -> PathBuf {
        if cfg!(target_os = "android") { 
            PathBuf::from_str("/data/data/com.example.Music/").unwrap()
        } else {
            dirs::config_dir().unwrap()
        }
    }

    pub fn load() -> Self {
        let file = std::fs::read_to_string(Self::dir().join("settings.toml")).unwrap_or_default();
        toml::from_str(&file).unwrap_or_default()
    }

    pub fn save(&self) {
        let file = toml::to_string(&self).unwrap();
        std::fs::write(Self::dir().join("settings.toml"), file);
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle
    }
}
