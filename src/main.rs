#![windows_subsystem = "windows"]

pub mod analysis;
pub mod app;
pub mod database;
pub mod gui;

use crate::app::audio::{AudioPlayer, self};
use crate::database::{init_db, row_to_weights};
use dioxus::{mobile::WindowBuilder, prelude::*};
use http::Response;
use log::{error, info};
use rusqlite::{params, Rows};
use std::collections::HashMap;
use std::io::Cursor;
use std::time::Instant;
use tracing_log::LogTracer;
#[cfg(target_os = "android")]
use crate::media::MediaMsg;
#[cfg(target_os = "android")]
use crate::media::MEDIA_MSG_TX;

#[cfg(not(target_os = "android"))]
use dioxus::desktop::use_asset_handler;
#[cfg(target_os = "android")]
use dioxus::mobile::use_asset_handler;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Notify;

use crate::app::controller::{MusicMsg, MUSIC_PLAYER_ACTIONS, controller, CONTROLLER_LOCK, UPDATE_CONTROLLER};
use std::sync::{Arc, Mutex};

use app::{
    settings::RadioSettings,
    track::{load_tracks, TrackInfo, get_track_image},
    MusicController,
};
use gui::*;

pub use gui::icons;

// CSS 
static MAIN_CSS: Asset = asset!("/assets/style.css");
static ALL_TRACKS_CSS: Asset = asset!("/assets/alltracks.css");
static EXPLORER_CSS: Asset = asset!("/assets/explorer.css");
static MENUBAR_CSS: Asset = asset!("/assets/menubar.css");
static QUEUE_CSS: Asset = asset!("/assets/queue.css");
static PLAYLISTS_CSS: Asset = asset!("/assets/playlists.css");
static SETTINGS_CSS: Asset = asset!("/assets/settings.css");
static TRACKOPTIONS_CSS: Asset = asset!("/assets/trackoptions.css");
static TRACKVIEW_CSS: Asset = asset!("/assets/trackview.css");

fn main() {
    // Hook panics into the logger to see them on android
    #[cfg(target_os = "android")]
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            info!("panic occurred: {s:?} at {:?}", panic_info.location());
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            info!("panic occurred: {s:?} at {:?}", panic_info.location());
        } else {
            info!("panic occurred");
        }
    }));

    UPDATE_CONTROLLER.set(Notify::new()).unwrap();
    let empty_controller = Arc::new(Mutex::new(MusicController::empty()));
    CONTROLLER_LOCK.set(empty_controller);

    std::thread::spawn(move || {
        use tokio::runtime::Runtime;

        let rt = Runtime::new().unwrap();
        let mut audio_player = AudioPlayer::new();
        
        let (tx, mut rx) = unbounded_channel();
        *MUSIC_PLAYER_ACTIONS.lock().unwrap() = Some(tx);

        rt.block_on(async {
            while let Some(msg) = rx.recv().await {
                match msg {
                    MusicMsg::Pause => audio_player.pause(),
                    MusicMsg::Play => audio_player.play(),
                    MusicMsg::Toggle => audio_player.toggle_playing(),
                    MusicMsg::PlayTrack(file) => audio_player.play_track(&file),
                    MusicMsg::SetVolume(volume) => audio_player.set_volume(volume),
                    MusicMsg::SetPos(pos) => audio_player.set_pos(pos),
                    _ => {}
                }
                controller().lock().unwrap().progress_secs = audio_player.progress_secs();
            }
        });
    });

    init();
}

#[cfg(target_os = "android")]
fn init() {
    use android_logger::Config;
    use log::LevelFilter;
    use env_filter::Builder;

    let mut builder = Builder::new();
    builder.filter(None, LevelFilter::Trace);
    builder.filter(Some("tungstenite"), LevelFilter::Off);

    let filter = builder.build();

    android_logger::init_once(
        Config::default()
            .with_filter(filter)
            .with_max_level(LevelFilter::Trace)
            .with_tag("com.example.Music"),
    );

    info!("Starting up trackfish");

    launch(SetUpRoute);
}

use dioxus::mobile::tao::window::Icon;

fn load_image() -> Icon {
    let png = &include_bytes!("../assets/icons/icon256.png")[..];
    let header = minipng::decode_png_header(png).expect("bad PNG");
    let mut buffer = vec![0; header.required_bytes_rgba8bpc()];
    let mut image = minipng::decode_png(png, &mut buffer).expect("bad PNG");
    image.convert_to_rgba8bpc().unwrap();
    let pixels = image.pixels();
    Icon::from_rgba(pixels.to_vec(), image.width(), image.height()).unwrap()
}

#[cfg(not(target_os = "android"))]
fn init() {
    LogTracer::init().expect("Failed to initialize LogTracer");

    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).unwrap();

    let window = WindowBuilder::new()
        .with_title("TrackFish")
        .with_always_on_top(false)
        .with_window_icon(Some(load_image()));
    let config = dioxus::desktop::Config::new().with_window(window);
    LaunchBuilder::new().with_cfg(config).launch(SetUpRoute);
}

#[component]
fn SetUpRoute() -> Element {
    use app::settings::Settings;
    let mut set_up = use_signal(Settings::exists);
    let mut dir = use_signal(Settings::default_audio_dir);

    rsx! {
        if set_up() {
            App {}
        } else {
            label { r#for: "directory", "Current directory: " }
            kbd { "{dir}" }
            br {}
            button {
                onclick: move |_| async move {
                    #[cfg(not(target_os = "android"))]
                    {
                        let file = rfd::FileDialog::new()
                            .set_directory("/")
                            .pick_folder();
                        if let Some(file) = file {
                            dir.set(file.display().to_string());
                        }
                    }
                },
                "Change Music Directory"
            }
            // Other options
            br {}
            br {}
            button {
                width: "200px",
                height: "50px",
                onclick: move |_| {
                    Settings {
                        directory: dir(),
                        volume: 1.0,
                        radio: RadioSettings::default(),
                    }
                        .save();
                    set_up.set(Settings::exists());
                },
                "Confirm"
            }
        }
    }
}

#[component]
fn App() -> Element {
    let mut loading_track_weights = use_signal(|| 0);
    let mut tracks_count = use_signal(|| 0);

    use_future(move || async move {
        std::thread::spawn(move || {
            let controller = CONTROLLER_LOCK.get().unwrap().clone();
            loop {
                let mut ctrl = controller.lock().unwrap();
                if ctrl.playing() && ctrl.song_length() - ctrl.progress_secs < 0.5 {
                    ctrl.skip();
                }

                if ctrl.playing() {
                    ctrl.progress_secs += 0.25;
                }

                drop(ctrl);
                std::thread::sleep(std::time::Duration::from_secs_f64(0.25));
            } 
        });
    });

    use_future(move || async move {
        let notify = UPDATE_CONTROLLER.get().unwrap().clone();
        loop {
            notify.notified().await;
            info!("updated");

            let controller = CONTROLLER_LOCK.get().unwrap().clone();
            let ctrl = controller.lock().unwrap().clone();
            CONTROLLER.set(ctrl);
            drop(controller);
        }
    });

    // Load in all tracks
    use_future(move || async move {
        let started = Instant::now();
        let controller = CONTROLLER_LOCK.get().unwrap();
        let mut ctrl = controller.lock().unwrap();

        let tracks = load_tracks(&ctrl.settings.directory);
        if let Ok(t) = tracks {
            tracks_count.set(t.len());
            let dir = ctrl.settings.directory.clone();
            *ctrl = MusicController::new(t, dir);
            info!("Loaded all tracks in {:?}", started.elapsed());
        } else {
            info!("{:?}", tracks);
        }

        let cache = init_db().unwrap();

        let mut stmt = cache.prepare("SELECT * FROM weights").unwrap();
        let mut result: Rows = stmt.query(params!()).unwrap();
        let mut weights: HashMap<String, TrackInfo> = HashMap::new();
        while let Ok(Some(row)) = result.next() {
            let hash = row.get(0).unwrap();
            weights.insert(hash, row_to_weights(&row).unwrap());
        }

        let len = ctrl.all_tracks.len();
        drop(ctrl);

        for i in 0..len {
            loading_track_weights += 1;
            let is_cached = controller.lock().unwrap().load_weight(&cache, &weights, i);
            if !is_cached {
                tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.001)).await;
            }
        }
    });

    // Start up media session
    #[cfg(target_os = "android")]
    use_future(move || async move {
        let (tx, mut rx) = unbounded_channel();
        *MEDIA_MSG_TX.lock().unwrap() = Some(tx);

        while let Some(msg) = rx.recv().await {
            match msg {
                MediaMsg::Play => controller().lock().unwrap().play(),
                MediaMsg::Pause => controller().lock().unwrap().pause(),
                MediaMsg::Next => controller().lock().unwrap().skip(),
                MediaMsg::Previous => controller().lock().unwrap().skipback(),
                MediaMsg::SeekTo(pos) => controller().lock().unwrap().set_pos(pos as f64 / 1000.0),
            }
        }
    });

    // Update mediasession as needed
    #[cfg(target_os = "android")]
    use_effect(move || {
        let controller = CONTROLLER_LOCK.get().unwrap().clone();
        let ctrl = controller.lock().unwrap();

        if let Some(track) = ctrl.current_track() {
            let image = get_track_image(&track.file);

            info!("Updating media notification");

            crate::gui::media::update_media_notification(
                &track.title,
                &track.artists[0],
                (track.len * 1000.0) as i64,
                (ctrl.progress_secs * 1000.0) as i64,
                ctrl.read().playing(),
                image).unwrap();
            drop(ctrl);
        }
    });

    use_asset_handler("trackimage",  |request, responder| {
        let r = Response::builder().status(404).body(&[]).unwrap();

        let id = if let Ok(id) = request.uri().path().replace("/trackimage/", "").parse() {
            id
        } else {
            responder.respond(r);
            return;
        };

        let track = controller().lock().unwrap().get_track(id).cloned();

        if track.is_none() {
            responder.respond(r);
            return;
        }

        let mut file = if let Some(file) = get_track_image(&track.unwrap().file) {
            Cursor::new(file)
        } else {
            responder.respond(r);
            return;
        };

        spawn(async move {
            match get_stream_response(&mut file, &request).await {
                Ok(response) => {
                    responder.respond(response);
                }
                Err(err) => error!("Error: {:?}", err),
            }
        });
    });

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        document::Stylesheet { href: ALL_TRACKS_CSS }
        document::Stylesheet { href: EXPLORER_CSS }
        document::Stylesheet { href: MENUBAR_CSS }
        document::Stylesheet { href: PLAYLISTS_CSS }
        document::Stylesheet { href: SETTINGS_CSS }
        document::Stylesheet { href: TRACKVIEW_CSS }
        document::Stylesheet { href: TRACKOPTIONS_CSS }
        document::Stylesheet { href: QUEUE_CSS }

        div {
            class: "loadingpopupbg",
            hidden: loading_track_weights() == tracks_count(),
            div { class: "loadingpopup",
                "Loading weights for track {loading_track_weights} out of {tracks_count} "
            }
        }

        div { class: "mainview", tabindex: 0, autofocus: true,
            TrackView {}
            TrackOptions {}
            QueueList {}
            AllTracks {}
            GenreList {}
            ArtistList {}
            AlbumsList {}
            PlaylistsView {}
            SearchView {}
            Settings {}
        }

        MenuBar {}
    }
}

#[component]
pub fn MenuBar() -> Element {
    rsx! {
        div { class: "buttonrow nav",
            button {
                class: "svg-button",
                background_image: "url({asset!(\"/assets/icons/song.svg\")})",
                onclick: move |_| VIEW.write().open(View::Song),
            }
            button {
                class: "svg-button",
                background_image: "url({asset!(\"/assets/icons/queue.svg\")})",
                onclick: move |_| VIEW.write().open(View::Queue),
            }
            button {
                class: "svg-button",
                background_image: "url({asset!(\"/assets/icons/folder.svg\")})",
                onclick: move |_| VIEW.write().open(View::AllTracks),
            }
            button {
                class: "svg-button",
                background_image: "url({asset!(\"/assets/icons/album.svg\")})",
                onclick: move |_| VIEW.write().open(View::Albums),
            }
            button {
                class: "svg-button",
                background_image: "url({asset!(\"/assets/icons/artist.svg\")})",
                onclick: move |_| VIEW.write().open(View::Artists),
            }
            button {
                class: "svg-button",
                background_image: "url({asset!(\"/assets/icons/genres.svg\")})",
                onclick: move |_| VIEW.write().open(View::Genres),
            }
            button {
                class: "svg-button",
                background_image: "url({asset!(\"/assets/icons/playlist.svg\")})",
                onclick: move |_| VIEW.write().open(View::Playlists),
            }
            button {
                class: "svg-button",
                background_image: "url({asset!(\"/assets/icons/search.svg\")})",
                onclick: move |_| VIEW.write().open(View::Search),
            }
            button {
                class: "svg-button",
                background_image: "url({asset!(\"/assets/icons/settings.svg\")})",
                onclick: move |_| VIEW.write().open(View::Settings),
            }
        }
    }
}
