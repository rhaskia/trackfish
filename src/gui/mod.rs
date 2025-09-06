pub mod confirm;
pub mod explorer;
pub mod playlists;
pub mod queuelist;
pub mod settings;
pub mod stream;
pub mod trackoptions;
pub mod trackview;
pub mod icons;

#[cfg(target_os = "android")]
pub mod media;

pub use icons::*;
pub use confirm::Confirmation;
pub use explorer::{AlbumsList, ArtistList, GenreList, AllTracks, SearchView};
pub use playlists::PlaylistsView;
pub use queuelist::QueueList;
pub use settings::Settings;
pub use stream::get_stream_response;
pub use trackoptions::TrackOptions;
pub use trackview::TrackView;
use crate::app::MusicController;
use dioxus::prelude::*;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::app::controller::{MusicMsg, PROGRESS_UPDATE, MUSIC_PLAYER_ACTIONS};
use log::info;
use std::sync::mpsc::channel;
use crate::app::audio::AudioPlayer;

pub const VIEW: GlobalSignal<ViewData> = Signal::global(|| ViewData::new());
pub const TRACKOPTION: GlobalSignal<Option<usize>> = Signal::global(|| None);
pub const ADD_TO_PLAYLIST: GlobalSignal<Option<usize>> = Signal::global(|| None);

pub static CONTROLLER: Lazy<Mutex<Option<SyncSignal<MusicController>>>> =
    Lazy::new(|| Mutex::new(None));

pub fn start_controller_thread() {
    std::thread::spawn(|| {
        let mut audio_player = AudioPlayer::new();
        
        let (music_tx, mut rx) = channel();
        *MUSIC_PLAYER_ACTIONS.lock().unwrap() = Some(music_tx);

        let (tx, mut progress_rx) = channel();
        *PROGRESS_UPDATE.lock().unwrap() = Some(progress_rx);

        audio_player.set_volume(0.0);

        info!("Started music message watcher");
        while let Ok(msg) = rx.recv() {
            info!("Recieved msg: {msg:?}");
            match msg {
                MusicMsg::Pause => audio_player.pause(),
                MusicMsg::Play => audio_player.play(),
                MusicMsg::Toggle => audio_player.toggle_playing(),
                MusicMsg::PlayTrack(file) => audio_player.play_track(&file),
                //MusicMsg::SetVolume(volume) => audio_player.set_volume(volume),
                MusicMsg::SetPos(pos) => audio_player.set_pos(pos),
                _ => {}
            }
            tx.send(audio_player.progress_secs()).unwrap();
        }
        info!("reciever failed");
    });

    std::thread::spawn(move || {
        if let Some(ctrl) = *CONTROLLER.lock().unwrap() {
            let mut controller = ctrl.clone();
            drop(ctrl);

            loop {
                if controller.read().playing() && controller.read().song_length() - controller.read().progress_secs < 0.5 {
                    controller.write().skip();
                }

                if controller.read().playing() {
                    controller.write().progress_secs += 0.25;
                }

                std::thread::sleep(std::time::Duration::from_secs_f64(0.25));
            } 
        }
    });
}

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
