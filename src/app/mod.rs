pub mod embed;
pub mod audio;
pub mod controller;
pub mod queue;
pub mod utils;
pub mod playlist;
pub mod track;
pub mod settings;

pub use controller::MusicController;
pub use track::{load_tracks, Track};
