use std::{path::PathBuf, str::FromStr};
use log::info;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Settings {
    pub volume: f32,
    pub directory: String,
    pub radio: RadioSettings,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct RadioSettings {
    pub temp: f32,
    pub album_penalty: f32,
    pub artist_penalty: f32,
    pub weight_mode: WeightMode,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Default)]
pub enum WeightMode {
    #[default]
    First,
    Average, 
    Last
}

impl Default for Settings {
    fn default() -> Self {
        let directory = if cfg!(target_os = "android") {
            "/storage/emulated/0/Music".to_string()
        } else { 
            match dirs::audio_dir() {
                Some(dir) => dir.display().to_string(),
                None => {
                    let dir = match std::env::consts::OS {
                        "linux" => "~/Music",
                        _ => ""
                    };
                    let _ = std::fs::create_dir(dir);
                    dir.to_string()
                }
            }
        };
        
        Self { 
            volume: 1.0,
            directory,
            radio: RadioSettings::default()
        }
    }
}

impl Default for RadioSettings {
    fn default() -> Self {
        Self {
            temp: 0.7,
            album_penalty: 0.2,
            artist_penalty: 0.7,
            weight_mode: WeightMode::default()
        }
    }
}

impl Settings {
    pub fn dir() -> PathBuf {
        if cfg!(target_os = "android") { 
            cache_dir::get_cache_dir().unwrap()
        } else {
            dirs::config_dir().unwrap().join("trackfish/")
        }
    }

    pub fn load() -> Self {
        let dir = Self::dir().join("settings.toml");
        info!("loading settings from {dir:?}");
        let file = std::fs::read_to_string(dir).unwrap_or_default();
        match toml::from_str(&file) {
            Ok(config) => config,
            Err(_) => {
                let config = Self::default();
                config.save();
                config
            }
        }
    }

    pub fn exists() -> bool {
        let dir = Self::dir().join("settings.toml");
        println!("{}", dir.exists());
        dir.exists()
    }

    pub fn save(&self) {
        let file = toml::to_string(&self).unwrap();
        std::fs::create_dir(Self::dir());
        std::fs::write(Self::dir().join("settings.toml"), file).unwrap();
    }
}
