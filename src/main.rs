pub mod app;
pub mod gui;
pub mod database;
pub mod analysis;

use dioxus::{prelude::*, dioxus_core::{SpawnIfAsync, LaunchConfig}, mobile::WindowBuilder};
use http::Response;
use log::{error, info};
use android_logger::Config;
use tracing_log::LogTracer;
use log::LevelFilter;
use id3::Tag;
use std::io::Cursor;
use std::ops::{AddAssign, SubAssign};
use std::time::Instant;

use crate::document::eval;

#[cfg(not(target_os = "android"))]
use dioxus::desktop::use_asset_handler;
#[cfg(target_os = "android")]
use dioxus::mobile::use_asset_handler;

use gui::*;
use app::{MusicController, audio::AudioPlayer, track::load_tracks};

fn main() {
    // Hook panics into the logger to see them on android
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
    android_logger::init_once(
        Config::default().with_max_level(LevelFilter::Trace).with_tag("com.example.Music"),
    );

    info!("Starting up trackfish");
    
    launch(App);
}

#[cfg(not(target_os = "android"))]
fn init() {
    LogTracer::init().expect("Failed to initialize LogTracer");

    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).unwrap();

    let window = WindowBuilder::new().with_always_on_top(false);
    let config = dioxus::desktop::Config::new().with_window(window);
    LaunchBuilder::new().with_cfg(config).launch(App);
}

#[component]
fn App() -> Element {
    let mut controller = use_signal(|| MusicController::empty());

    use_future(|| async {
        match eval(include_str!("../js/mediasession.js")).await {
            Ok(_) => {},
            Err(err) => log::error!("{err:?}"),
        }
    });

    use_memo(move || {
        info!("{:?}", VIEW.read().current);
    });

    use_future(move || async move { 
        let started = Instant::now();
        let result = crossbow::Permission::StorageRead.request_async().await;
        info!("{result:?}");

        let tracks = load_tracks(&controller.read().settings.directory);
        if let Ok(t) = tracks {
            if let Ok(mut c) = controller.try_write() {
                *c = MusicController::new(t, c.settings.directory.clone());
            } else {
                info!("Controller already borrowed");
            }
            info!("Loaded tracks in {:?}", started.elapsed());
        } else {
            info!("{:?}", tracks);
        }
    });

    use_asset_handler("trackimage", move |request, responder| {
        let r = Response::builder().status(200).body(&[]).unwrap();

        let id = if let Ok(parsed) = request.uri().path().replace("/trackimage/", "").parse() {
            parsed
        } else { responder.respond(r); return };

        // Retry once free
        let path = if let Ok(Some(track)) = controller.try_read().and_then(|c| Ok(c.get_track(id).cloned())) { 
            track.file
        } else { responder.respond(r); return };

        let path = format!("{}/{path}", controller.read().settings.directory);

        let tag = if let Ok(t) = Tag::read_from_path(path) {
            t
        } else { responder.respond(r); return };

        let mut file = if let Some(picture) = tag.pictures().next() {
            Cursor::new(picture.data.clone())
        } else { responder.respond(r); return };

        spawn(async move {
            match get_stream_response(&mut file, &request).await {
                Ok(response) => responder.respond(response),
                Err(err) => error!("Error: {:?}", err),
            }
        });
    });

    rsx! {
        document::Link { href: "assets/style.css", rel: "stylesheet" }

        document::Link { href: "assets/alltracks.css", rel: "stylesheet" }
        document::Link { href: "assets/explorer.css", rel: "stylesheet" }
        document::Link { href: "assets/menubar.css", rel: "stylesheet" }
        document::Link { href: "assets/settings.css", rel: "stylesheet" }
        document::Link { href: "assets/trackview.css", rel: "stylesheet" }

        div { class: "mainview",
            tabindex: 0,
            autofocus: true,
            onkeydown: move |e| match e.data().key() {
                Key::Character(c) => match c.as_str() {
                    "L" => VIEW.write().current.shift_down(),
                    "H" => VIEW.write().current.shift_up(),
                    _ => {}
                },
                _ => {}
            },
            TrackView { controller }
            QueueList { controller }
            AllTracks { controller }
            // GenreList { controller }
            // ArtistList { controller }
            // AlbumsList { controller }
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
                class: "songview-button",
                class: "svg-button",
                onclick: move |_| VIEW.write().open(View::Song),
            }
            button {
                class: "queue-button",
                class: "svg-button",
                onclick: move |_| VIEW.write().open(View::Queue),
            }
            button {
                class: "alltracks-button",
                class: "svg-button",
                onclick: move |_| VIEW.write().open(View::AllTracks),
            }
            button {
                class: "album-button",
                class: "svg-button",
                onclick: move |_| VIEW.write().open(View::Albums),
            }
            button {
                class: "artist-button",
                class: "svg-button",
                onclick: move |_| VIEW.write().open(View::Artists),
            }
            button {
                class: "genres-button",
                class: "svg-button",
                onclick: move |_| VIEW.write().open(View::Genres),
            }
            button {
                class: "search-button",
                class: "svg-button",
                onclick: move |_| VIEW.write().open(View::Search),
            }
            button {
                class: "settings-button",
                class: "svg-button",
                onclick: move |_| VIEW.write().open(View::Settings),
            }
        }
    }
}
