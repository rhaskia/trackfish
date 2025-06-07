use dioxus::prelude::*;
use app::MusicController;

pub const VIEW: GlobalSignal<ViewData> = Signal::global(|| ViewData::new());
pub const CONTROLLER: GlobalSignal<MusicController> =
    GlobalSignal::new(|| MusicController::empty());
pub const TRACKOPTION: GlobalSignal<Option<usize>> = Signal::global(|| None);
pub const ADD_TO_PLAYLIST: GlobalSignal<Option<usize>> = Signal::global(|| None);

#[derive(Debug, PartialEq, Clone)]
pub enum View {
    Song = 0,
    Queue = 1,
    AllTracks = 2,
    Artists = 3,
    Genres = 4,
    Albums = 5,
    Playlists = 6,
    Search = 7,
    Settings = 8,
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
            6 => Self::Playlists,
            7 => Self::Search,
            8 => Self::Settings,
            _ => Self::Song,
        }
    }
}

pub struct ViewData {
    pub current: View,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub playlist: Option<usize>,
    pub genre: Option<String>,
}

impl ViewData {
    pub fn new() -> Self {
        Self {
            current: View::Song,
            album: None,
            artist: None,
            genre: None,
            playlist: None,
        }
    }

    pub fn open(&mut self, view: View) {
        self.current = view;
    }
}
