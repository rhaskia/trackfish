//#![feature(trivial_bounds)]

pub mod app;
pub mod gui;

use dioxus::prelude::*;
use log::error;

use log::info;
use android_logger::Config;
use tracing_log::LogTracer;
use log::LevelFilter;
use id3::Tag;
use std::io::Cursor;

use crate::document::eval;

#[cfg(not(target_os = "android"))]
use dioxus::desktop::use_asset_handler;
#[cfg(target_os = "android")]
use dioxus::mobile::use_asset_handler;

use app::*;
use gui::*;

fn main() {
    if cfg!(target_os = "android") {
       android_logger::init_once(
            Config::default().with_max_level(LevelFilter::Trace),
       );
    }

    LogTracer::init().expect("Failed to initialize LogTracer");
    
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO);

    launch(App);
}

const DIR: GlobalSignal<&str> = GlobalSignal::new(|| {
    if cfg!(target_os = "android") {
        "/storage/emulated/0/Music/"
    } else {
        "E:/Music/"
    }
});

const TRACKS: GlobalSignal<Vec<Track>> = GlobalSignal::new(|| Vec::new());

#[component]
fn App() -> Element {
    let mut controller = use_signal(|| MusicController::new(Vec::new()));
    let mut view = use_signal(|| View::Song);

    use_future(|| async {
        eval(include_str!("../js/mediasession.js")).await;
    });

    use_future(move || async move { 
        info!("Requested storage permissions: {:?}", crossbow::Permission::StorageWrite.request_async().await);
        controller.set(MusicController::new(load_tracks(DIR())));
        info!("loaded all tracks into music controller");
    });

    use_asset_handler("trackimage", move |request, responder| {
        let id = request.uri().path().replace("/trackimage/", "").parse().unwrap();
        let path = if let Some(track) = controller.read().get_track(id) { 
            track.file.clone()
        } else {
            return;
        };
        let tag = Tag::read_from_path(path).unwrap();
        let mut file = if let Some(picture) = tag.pictures().next() {
            Cursor::new(picture.data.clone())
        } else { return };

        tokio::task::spawn(async move {
            match get_stream_response(&mut file, &request).await {
                Ok(response) => responder.respond(response),
                Err(err) => eprintln!("Error: {}", err),
            }
        });
    });

    rsx! {
        style {{ include_str!("../assets/style.css") }}

        div {
            class: "mainview",
            match &*view.read() {
                View::Song => rsx!{ TrackView { controller } },
                View::Queue => rsx!{ QueueList { controller } },
                View::AllTracks => rsx!{ AllTracks { controller } },
                View::Genres => rsx!{ GenreList { controller } },
                View::Artists => rsx!{ ArtistList { controller } },
                View::Albums => rsx!{ AlbumsList { controller } },
                View::Settings => rsx!{ Settings { controller } },
                View::Search => rsx!{},
            }
        }

        MenuBar { view }
    }
}



#[component]
pub fn MenuBar(view: Signal<View>) -> Element {
    rsx! {
        div {
            class: "buttonrow nav",
            button {
                class: "songview-button",
                class: "svg-button",
                onclick: move |_| view.set(View::Song),
            }
            button {
                class: "queue-button",
                class: "svg-button",
                onclick: move |_| view.set(View::Queue),
            }
            button {
                class: "alltracks-button",
                class: "svg-button",
                onclick: move |_| view.set(View::AllTracks),
            }
            button {
                class: "album-button",
                class: "svg-button",
                onclick: move |_| view.set(View::Albums),
            }
            button {
                class: "artist-button",
                class: "svg-button",
                onclick: move |_| view.set(View::Artists),
            }
            button {
                class: "genres-button",
                class: "svg-button",
                onclick: move |_| view.set(View::Genres),
            }
            button {
                class: "search-button",
                class: "svg-button",
                onclick: move |_| view.set(View::Search),
            }
            button {
                class: "settings-button",
                class: "svg-button",
                onclick: move |_| view.set(View::Settings),
            }
        }

    }
}

pub enum View {
    Song, 
    Queue,
    AllTracks,
    Artists,
    Genres,
    Albums,
    Search,
    Settings,
}
