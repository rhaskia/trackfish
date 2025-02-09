//#![feature(trivial_bounds)]

pub mod app;
pub mod gui;
pub mod database;

use dioxus::{prelude::*, dioxus_core::SpawnIfAsync};
use http::Response;
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

use gui::*;
use app::{MusicController, audio::AudioPlayer, track::load_tracks};

const VIEW: GlobalSignal<ViewData> = Signal::global(|| ViewData::new());

fn main() {
    if cfg!(target_os = "android") {
        android_logger::init_once(
            Config::default().with_max_level(LevelFilter::Trace).with_tag("com.example.Music"),
        );
    } else {
        LogTracer::init().expect("Failed to initialize LogTracer");

        dioxus_logger::init(dioxus_logger::tracing::Level::INFO).unwrap();
    }
    
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            info!("panic occurred: {s:?} at {:?}", panic_info.location());
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            info!("panic occurred: {s:?} at {:?}", panic_info.location());
        } else {
            info!("panic occurred");
        }
    }));

    launch(App);
}

const DIR: GlobalSignal<&str> = GlobalSignal::new(|| {
    if cfg!(target_os = "android") {
        "/storage/emulated/0/Music/"
    } else {
        "E:/Music/"
    }
});

#[component]
fn App() -> Element {
    let mut controller = use_signal(|| MusicController::new(Vec::new()));

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
        let result = crossbow::Permission::StorageRead.request_async().await;
        info!("{result:?}");

        let tracks = load_tracks(&controller.read().settings.directory);
        if let Ok(t) = tracks {
            if let Ok(mut c) = controller.try_write() {
                *c = MusicController::new(t);
            } else {
                info!("Controller already borrowed");
            }
            info!("loaded all tracks into music controller");
        } else {
            info!("{:?}", tracks);
        }
    });

    use_asset_handler("trackimage", move |request, responder| {
        let r = Response::builder().status(404).body(&[]).unwrap();

        let id = if let Ok(parsed) = request.uri().path().replace("/trackimage/", "").parse() {
            parsed
        } else { responder.respond(r); return };

        // Retry once free
        let path = if let Ok(Some(track)) = controller.try_read().and_then(|c| Ok(c.get_track(id).cloned())) { 
            track.file
        } else { responder.respond(r); return };

        let tag = if let Ok(t) = Tag::read_from_path(path) {
            t
        } else { responder.respond(r); return };

        let mut file = if let Some(picture) = tag.pictures().next() {
            Cursor::new(picture.data.clone())
        } else { responder.respond(r); return };

        spawn(async move {
            match get_stream_response(&mut file, &request).await {
                Ok(response) => responder.respond(response),
                Err(err) => eprintln!("Error: {}", err),
            }
        });
    });

    rsx! {
        style { {include_str!("../assets/style.css")} }

        div { class: "mainview",
            match &VIEW.read().current {
                View::Song => rsx! {
                    TrackView { controller }
                },
                View::Queue => rsx! {
                    QueueList { controller }
                },
                View::AllTracks => rsx! {
                    AllTracks { controller }
                },
                View::Genres => rsx! {
                    GenreList { controller }
                },
                View::Artists => rsx! {
                    ArtistList { controller }
                },
                View::Albums => rsx! {
                    AlbumsList { controller }
                },
                View::Settings => rsx! {
                    Settings { controller }
                },
                _ => rsx! {},
            }
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

#[derive(Debug, PartialEq, Clone)]
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

pub struct ViewData {
    pub current: View,
    pub album: Option<usize>,
    pub artist: Option<usize>,
    pub genre: Option<usize>
}

impl ViewData {
    pub fn new() -> Self {
        Self { current: View::Song, album: None, artist: None, genre: None }
    }

    pub fn open(&mut self, view: View) {
        self.current = view;
    }
}
