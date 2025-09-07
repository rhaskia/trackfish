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
use app::controller::{MusicMsg, PROGRESS_UPDATE}; 
use crate::app::controller::MUSIC_PLAYER_ACTIONS;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use std::sync::mpsc::channel;

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
    let mut controller = use_signal_sync(|| MusicController::empty());
    *gui::CONTROLLER.lock().unwrap() = Some(controller);

    #[cfg(not(target_os = "android"))]
    use_future(move || async move {
        crate::gui::start_controller_thread();
    });

    // Load in all tracks
    use_future(move || async move {
        let started = Instant::now();

        let tracks = load_tracks(&controller.read().settings.directory);
        if let Ok(t) = tracks {
            tracks_count.set(t.len());
            let dir = controller.read().settings.directory.clone();
            controller.set(MusicController::new(t, dir));
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

        let len = controller.read().all_tracks.len();

        for i in 0..len {
            loading_track_weights += 1;
            let is_cached = controller.write().load_weight(&cache, &weights, i);
            if !is_cached {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }
        }
    });

    use_asset_handler("trackimage",  move |request, responder| {
        let r = Response::builder().status(404).body(&[]).unwrap();

        let id = if let Ok(id) = request.uri().path().replace("/trackimage/", "").parse() {
            id
        } else {
            responder.respond(r);
            return;
        };

        let track = controller.read().get_track(id).cloned();

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
            TrackView { controller }
            TrackOptions { controller }
            QueueList { controller }
            AllTracks { controller }
            GenreList { controller }
            ArtistList { controller }
            AlbumsList { controller }
            PlaylistsView { controller }
            SearchView { controller }
            Settings { controller }
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
