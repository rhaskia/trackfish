pub mod all_tracks;
pub mod queuelist;
pub mod settings;
pub mod trackview;
pub mod stream;
pub mod explorer;
pub mod input;
#[cfg(target_os = "android")]
pub mod media;

pub use stream::get_stream_response;
pub use settings::Settings;
pub use all_tracks::AllTracks;
pub use trackview::TrackView;
pub use queuelist::QueueList;
pub use explorer::{AlbumsList, ArtistList, GenreList};
pub use input::{key_to_action, Action};

use dioxus::prelude::*;
use crate::app::MusicController;

pub const VIEW: GlobalSignal<ViewData> = Signal::global(|| ViewData::new());
pub const CONTROLLER: GlobalSignal<MusicController> = GlobalSignal::new(|| MusicController::empty());

#[derive(Debug, PartialEq, Clone)]
pub enum View {
    Song = 0, 
    Queue = 1,
    AllTracks = 2,
    Artists = 3,
    Genres = 4,
    Albums = 5,
    Search = 6,
    Settings = 7,
}

impl View {
    pub fn shift_up(&mut self) {
        *self = Self::from_usize(self.clone() as usize + 1);
    } 

    pub fn shift_down(&mut self) {
        // No overflows
        if *self == Self::Song { 
            *self = Self::Settings;
            return;
        }
        *self = Self::from_usize(self.clone() as usize - 1);
    } 

    fn from_usize(n: usize) -> Self {
        match n {
            0 => Self::Song,
            1 => Self::Queue,
            2 => Self::AllTracks,
            3 => Self::Artists,
            4 => Self::Genres,
            5 => Self::Albums,
            6 => Self::Search,
            7 => Self::Settings,
            _ => Self::Song,
        }
    }
}

pub struct ViewData {
    pub current: View,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub genre: Option<String>
}

impl ViewData {
    pub fn new() -> Self {
        Self { current: View::Song, album: None, artist: None, genre: None }
    }

    pub fn open(&mut self, view: View) {
        self.current = view;
    }
}
