pub mod analysis;
pub mod app;
pub mod database;
pub mod gui;

use dioxus::{mobile::WindowBuilder, prelude::*};
use http::Response;
use id3::Tag;
use log::{error, info};
use std::io::Cursor;
use std::time::Instant;
use tracing_log::LogTracer;
use std::collections::HashMap;
use crate::database::{row_to_weights, init_db};
use rusqlite::{Rows, params};

use crate::document::eval;

#[cfg(not(target_os = "android"))]
use dioxus::desktop::use_asset_handler;
#[cfg(target_os = "android")]
use dioxus::mobile::use_asset_handler;

use app::{track::{load_tracks, TrackInfo}, MusicController, settings::RadioSettings};
use gui::*;

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
    use android_logger::Config;
    use log::LevelFilter;

    android_logger::init_once(
        Config::default().with_max_level(LevelFilter::Trace).with_tag("com.example.Music"),
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
    let dir = use_signal(Settings::default_audio_dir);

    rsx! {
        if set_up() {
            App {}
        } else {
            label {
                r#for: "directory",
                "Select music directory:"
            }
            br {}
            input { 
                id: "directory",
                value: dir,
            }
            // Other options
            br {}
            br {}
            button {
                onclick: move |_| { 
                    Settings { directory: dir(), volume: 1.0, radio: RadioSettings::default() }.save();
                    set_up.set(true);
                },
                "Confirm"
            }
        }
    }
}

#[component]
fn App() -> Element {
    let mut controller = use_signal(|| MusicController::empty());
    let mut loading_track_weights = use_signal(|| 0);
    let mut tracks_count = use_signal(|| 0);

    use_future(|| async {
        match eval(include_str!("../js/mediasession.js")).await {
            Ok(_) => {}
            Err(err) => log::error!("{err:?}"),
        }
    });

    use_memo(move || {
        info!("{:?}", VIEW.read().current);
    });

    use_future(move || async move {
        let started = Instant::now();
        info!("hi");
        #[cfg(target_os = "android")]
        {
            //let result = crossbow::Permission::StorageRead.request_async().await;
            let result = crossbow_android::permission::request_permission(&crossbow_android::permission::AndroidPermission::ReadMediaAudio).await;
            info!("{result:?}");
        }

        info!("hi");
        let tracks = load_tracks(&controller.read().settings.directory);
        if let Ok(t) = tracks {
            tracks_count.set(t.len());
            if let Ok(mut c) = controller.try_write() {
                *c = MusicController::new(t, c.settings.directory.clone());
            } else {
                info!("Controller already borrowed");
            }
            info!("Loaded tracks in {:?}", started.elapsed());
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
                tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.001)).await;
            }
        }
    });

    use_future(|| async {
        #[cfg(target_os = "android")]
        {
            let result = crossbow_android::permission::request_permission(&crossbow_android::permission::AndroidPermission::PostNotifications).await;
            info!("{result:?}");
            crate::gui::media::set_media_context();
            info!("silly");
        }
    });

    use_asset_handler("trackimage", move |request, responder| {
        let r = Response::builder().status(200).body(&[]).unwrap();

        let id = if let Ok(parsed) = request.uri().path().replace("/trackimage/", "").parse() {
            parsed
        } else {
            responder.respond(r);
            return;
        };

        // Retry once free
        let path = if let Ok(Some(track)) =
            controller.try_read().and_then(|c| Ok(c.get_track(id).cloned()))
        {
            track.file
        } else {
            responder.respond(r);
            return;
        };

        let tag = if let Ok(t) = Tag::read_from_path(path) {
            t
        } else {
            responder.respond(r);
            return;
        };

        let mut file = if let Some(picture) = tag.pictures().next() {
            Cursor::new(picture.data.clone())
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
        document::Link { href: "assets/style.css", rel: "stylesheet" }

        document::Link { href: "assets/alltracks.css", rel: "stylesheet" }
        document::Link { href: "assets/explorer.css", rel: "stylesheet" }
        document::Link { href: "assets/menubar.css", rel: "stylesheet" }
        document::Link { href: "assets/settings.css", rel: "stylesheet" }
        document::Link { href: "assets/trackview.css", rel: "stylesheet" }
        document::Link { href: "assets/queue.css", rel: "stylesheet" }
        
        div {
            class: "loadingpopup",
            hidden: loading_track_weights() == tracks_count(),
            "Loading weights for track {loading_track_weights} out of {tracks_count} "
        }

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
            //AllTracks { controller }
            GenreList { controller }
            ArtistList { controller }
            AlbumsList { controller }
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
            // button {
            //     class: "search-button",
            //     class: "svg-button",
            //     onclick: move |_| VIEW.write().open(View::Search),
            // }
            button {
                class: "settings-button",
                class: "svg-button",
                onclick: move |_| VIEW.write().open(View::Settings),
            }
        }
    }
}
