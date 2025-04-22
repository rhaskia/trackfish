use log::info;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Settings {
    pub volume: f32,
    pub directory: String,
    pub radio: RadioSettings,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct RadioSettings {
    pub temp: f32,
    pub weight_mode: WeightMode,
    pub album_penalty: f32,
    pub artist_penalty: f32,

    pub mfcc_weight: f32,
    pub chroma_weight: f32,
    pub spectral_weight: f32,
    pub energy_weight: f32,
    pub bpm_weight: f32,
    pub zcr_weight: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Default)]
pub enum WeightMode {
    #[default]
    First,
    Average,
    Last,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            directory: Self::default_audio_dir(),
            radio: RadioSettings::default(),
        }
    }
}

impl Default for RadioSettings {
    fn default() -> Self {
        Self {
            temp: 0.7,
            album_penalty: 0.2,
            artist_penalty: 0.7,
            weight_mode: WeightMode::default(),
            mfcc_weight: 1.0,
            chroma_weight: 1.0,
            spectral_weight: 0.7,
            energy_weight: 0.0,
            bpm_weight: 0.0,
            zcr_weight: 0.0,
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

    pub fn default_audio_dir() -> String {
        if cfg!(target_os = "android") {
            "/storage/emulated/0/Music".to_string()
        } else {
            match dirs::audio_dir() {
                Some(dir) => dir.display().to_string(),
                None => {
                    let dir = match std::env::consts::OS {
                        "linux" => "~/Music",
                        _ => "",
                    };
                    let _ = std::fs::create_dir(dir);
                    dir.to_string()
                }
            }
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
        if let Err(err) = std::fs::create_dir(Self::dir()) {
            info!("{err}");
        };
        std::fs::write(Self::dir().join("settings.toml"), file).unwrap();
    }
}
