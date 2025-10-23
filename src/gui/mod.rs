pub mod confirm;
pub mod explorer;
pub mod icons;
pub mod playlists;
pub mod queuelist;
pub mod settings;
pub mod stream;
pub mod trackoptions;
pub mod trackview;

#[cfg(target_os = "android")]
pub mod media;
#[cfg(target_os = "android")]
use crate::gui::media::{MediaMsg, MEDIA_MSG_TX};

use dioxus::prelude::*;
use log::info;
use once_cell::sync::Lazy;
use std::sync::mpsc::channel;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::Mutex;

use crate::app::audio::AudioPlayer;
use crate::app::controller::{MusicMsg, MUSIC_PLAYER_ACTIONS};
use crate::app::track::get_track_image;
use crate::app::MusicController;

pub use confirm::Confirmation;
pub use explorer::{AlbumsList, AllTracks, ArtistList, GenreList, SearchView};
pub use icons::*;
pub use playlists::PlaylistsView;
pub use queuelist::QueueList;
pub use settings::Settings;
pub use stream::get_stream_response;
pub use trackoptions::TrackOptions;
pub use trackview::TrackView;

/// Current view of the application, eg TrackView, Queue, Settings, etc
pub const VIEW: GlobalSignal<ViewData> = Signal::global(|| ViewData::new());
/// If a track options menu is currently open (a Some value containing the track ID) or not (None)
pub const TRACKOPTION: GlobalSignal<Option<usize>> = Signal::global(|| None);
/// To be set when a song is to be added to the playlist by a user
pub const ADD_TO_PLAYLIST: GlobalSignal<Option<usize>> = Signal::global(|| None);

pub const MOBILE: GlobalSignal<bool> = Signal::global(|| cfg!(target_os = "android"));

/// Global reference to the dioxus SyncSignal holding the main MusicController
/// This allows the controller to be used in threads, and from outside a component
pub static CONTROLLER: Lazy<Mutex<Option<SyncSignal<MusicController>>>> =
    Lazy::new(|| Mutex::new(None));

/// Starts a thread running all background tasks for the MusicController
/// To avoid issues on Android where the app freezes in the background, this allows the app to
/// run from a foreground service initiated runtime
pub fn start_controller_thread() {
    std::thread::spawn(|| {
        let res = std::panic::catch_unwind(|| {
            let mut track_playing = false;
            let mut audio_player = AudioPlayer::new();

            #[allow(unused_mut)]
            let (music_tx, mut rx) = channel();
            *MUSIC_PLAYER_ACTIONS.lock().unwrap() = Some(music_tx);

            #[allow(unused_mut)]
            #[cfg(target_os = "android")]
            let (tx, mut media_rx) = channel();
            #[cfg(target_os = "android")]
            {
                *MEDIA_MSG_TX.lock().unwrap() = Some(tx);
            }

            info!("Started music message watcher");

            loop {
                // Watches for Android media notification callbacks
                // Possibly could move these into callback functions themselves to cut out the
                // middle man
                #[cfg(target_os = "android")]
                while let Ok(msg) = media_rx.try_recv() {
                    if let Some(ctrl) = *CONTROLLER.lock().unwrap() {
                        let mut controller = ctrl.clone();

                        match msg {
                            MediaMsg::Play => controller.write().play(),
                            MediaMsg::Pause => controller.write().pause(),
                            MediaMsg::Next => controller.write().skip(),
                            MediaMsg::Previous => controller.write().skipback(),
                            MediaMsg::SeekTo(pos) => {
                                controller.write().set_pos(pos as f64 / 1000.0)
                            }
                        }
                    }
                }

                // Watches for AudioPlayer messages
                // AudioPlayer is not Sync, so it has to stay within the thread
                match rx.recv_timeout(std::time::Duration::from_millis(50)) {
                    Ok(msg) => {
                        info!("Recieved msg: {msg:?}");
                        match msg {
                            MusicMsg::Pause => audio_player.pause(),
                            MusicMsg::Play => audio_player.play(),
                            MusicMsg::Toggle => audio_player.toggle_playing(),
                            MusicMsg::PlayTrack(file) => {
                                if let Some(ctrl) = *CONTROLLER.lock().unwrap() {
                                    let mut controller = ctrl.clone();
                                    controller.write().song_length = audio_player.play_track(&file);
                                    controller.write().progress_secs = 0.0;

                                    track_playing = true;
                                }
                            }
                            MusicMsg::SetVolume(volume) => audio_player.set_volume(volume),
                            MusicMsg::SetPos(pos) => audio_player.set_pos(pos),
                            _ => {}
                        }

                        if let Some(ctrl) = *CONTROLLER.lock().unwrap() {
                            let mut controller = ctrl.clone();
                            controller.write().progress_secs = audio_player.progress_secs();
                            controller.write().playing = audio_player.playing();

                            let track = controller.read().current_track().cloned();

                            // Set media notification to update user and keep FGS alive
                            #[cfg(target_os = "android")]
                            if let Some(track) = track {
                                let image = get_track_image(&track.file);

                                info!("Updating media notification");
                                // Avoid accessing the controller twice in a statement, as the app
                                // seems to freak out about it 
                                let progress = (controller.read().progress_secs * 1000.0) as i64;

                                let result = crate::gui::media::update_media_notification(
                                    &track.title,
                                    &track.artists[0],
                                    (track.len * 1000.0) as i64,
                                    progress,
                                    controller.read().playing(),
                                    image,
                                );
                                info!("Media notification result: {result:?}");
                            }
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        info!("Channel disconnected");
                        break;
                    } // channel closed
                    _ => {}
                }

                // Manage track skipping
                if audio_player.track_ended() && track_playing {
                    if let Some(ctrl) = *CONTROLLER.lock().unwrap() {
                        let mut controller = ctrl.clone();
                        controller.write().skip();
                        track_playing = false;
                    }
                }
            }
        });

        if let Err(e) = res {
            log::error!("Music thread panicked: {:?}", e);
        }
    });
}

/// Enum holding view state
#[derive(Debug, PartialEq, Clone)]
pub enum View {
    Song = 0,
    Queue = 1,
    AllTracks = 2,
    Albums = 3,
    Artists = 4,
    Genres = 5,
    Playlists = 6,
    Search = 7,
    Settings = 8,
}

impl View {
    /// Shifts the view up in number (moves to the right)
    pub fn shift_up(&mut self) {
        *self = Self::from_usize(self.clone() as usize + 1);
    }

    /// Shifts the view down number (moves to the left)
    pub fn shift_down(&mut self) {
        // No overflows
        if *self == Self::Song {
            *self = Self::Settings;
            return;
        }
        *self = Self::from_usize(self.clone() as usize - 1);
    }

    /// Turns a number value into a view state
    pub fn from_usize(n: usize) -> Self {
        match n {
            0 => Self::Song,
            1 => Self::Queue,
            2 => Self::AllTracks,
            3 => Self::Albums,
            4 => Self::Artists,
            5 => Self::Genres,
            6 => Self::Playlists,
            7 => Self::Settings,
            _ => Self::Song,
        }
    }
}

/// Holds the current view, and information about if track views are open
pub struct ViewData {
    pub current: View,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub playlist: Option<usize>,
    pub autoplaylist: Option<usize>,
    pub genre: Option<String>,
}

impl ViewData {
    /// Create a new ViewData with no open trackviews on Song view
    pub fn new() -> Self {
        Self {
            current: View::Song,
            album: None,
            artist: None,
            genre: None,
            playlist: None,
            autoplaylist: None,
        }
    }

    /// Opens a View
    pub fn open(&mut self, view: View) {
        if view == self.current {
            match view {
                View::Artists => self.artist = None,
                View::Genres => self.genre = None,
                View::Albums => self.album = None,
                View::Playlists => {
                    self.autoplaylist = None;
                    self.playlist = None;
                },
                _ => {}
            }
        }
        self.current = view;
    }
}
