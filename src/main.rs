#![windows_subsystem = "windows"]
#![allow(const_item_mutation)]

pub mod analysis;
pub mod app;
pub mod database;
pub mod gui;

use std::io::Cursor;

#[cfg(target_os="android")]
use dioxus::mobile::{use_wry_event_handler, use_asset_handler};
#[cfg(not(target_os = "android"))]
use dioxus::desktop::{use_asset_handler, WindowBuilder};
#[cfg(not(target_os = "android"))]
use tracing_log::LogTracer;
use dioxus:: prelude::*;
use http::Response;
use log::info;
use dioxus::document::eval;
use app::{
    track::get_track_image,
    MusicController,
};

use gui::*;
pub use gui::icons;

// CSS
static MAIN_CSS: Asset = asset!("/assets/style.css");
static ALL_TRACKS_CSS: Asset = asset!("/assets/alltracks.css");
static AUTOPLAYLISTS_CSS: Asset = asset!("/assets/autoplaylists.css");
static EXPLORER_CSS: Asset = asset!("/assets/explorer.css");
static LIBRARYMANAGEMENT_CSS: Asset = asset!("/assets/librarymanagement.css");
static MENUBAR_CSS: Asset = asset!("/assets/menubar.css");
static QUEUE_CSS: Asset = asset!("/assets/queue.css");
static PLAYLISTS_CSS: Asset = asset!("/assets/playlists.css");
static SETTINGS_CSS: Asset = asset!("/assets/settings.css");
static TAGEDITOR_CSS: Asset = asset!("/assets/tageditor.css");
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
    use env_filter::Builder;
    use log::LevelFilter;

    let mut builder = Builder::new();
    builder.filter(None, LevelFilter::Trace);
    builder.filter(Some("tungstenite"), LevelFilter::Off);
    builder.filter(Some("jni"), LevelFilter::Off);
    builder.filter(Some("symphonia_bundle_mp3"), LevelFilter::Off);
    builder.filter(Some("symphonia_core"), LevelFilter::Off);
    builder.filter(Some("symphonia_core"), LevelFilter::Off);
    builder.filter(Some("symphonia_metadata"), LevelFilter::Off);

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

#[cfg(not(target_os = "android"))]
use dioxus::mobile::tao::window::Icon;

use crate::gui::stream::get_stream_response;

#[cfg(not(target_os = "android"))]
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
    LogTracer::builder()
        .ignore_crate("symphonia_core")
        .ignore_crate("symphonia_metadata")
        .ignore_crate("symphonia_bundle_mp3")
        .ignore_crate("symphonia")
        .init().expect("Failed to initialize LogTracer");

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
    #[allow(unused_mut)]
    let mut dir = use_signal(Settings::default_audio_dir);

    rsx! {
        if set_up() {
            App {}
        } else {
            // spacer for android status bar area
            if cfg!(target_os = "android") {
                div { height: "30pt" }
            }

            label { r#for: "directory", "Current directory: " }
            kbd { "{dir}" }
            br {}
            button {
                onclick: move |_| async move {
                    #[cfg(not(target_os = "android"))]
                    {
                        let file = rfd::FileDialog::new().set_directory("/").pick_folder();
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
                        ..Default::default()
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
    let loading_track_weights = use_signal(|| 0);
    let tracks_count = use_signal(|| 0);
    let mut controller = use_signal_sync(|| MusicController::empty());
    *gui::CONTROLLER.lock().unwrap() = Some(controller);
    
    let mut handle = use_signal(|| None);

    #[cfg(not(target_os = "android"))]
    use_future(move || async move {
        crate::gui::start_controller_thread();
    });

    use_future(move || async move {
        let res = crate::gui::init_tracks();

        handle.set(Some(res));
    });

    use_effect(move || {
        let scroll_index = VIEW.read().current.clone() as usize; 
        eval(&format!(r#"
            const mainview = document.getElementById("mainview");
            const scrollWidth = mainview.scrollWidth;
            mainview.scrollLeft = scrollWidth / 9 * {scroll_index}
        "#));
    });

    // Attach on scroll snap change to main view
    use_future(move || async move {
        let mut js = eval(r#"
            const mainview = document.getElementById("mainview");
            mainview.addEventListener("scrollsnapchange", (event) => {
                console.log(mainview.scrollLeft / mainview.scrollWidth * 9);
                dioxus.send(mainview.scrollLeft / mainview.scrollWidth * 9);
            });
        "#);  

        while let Ok(res) = js.recv::<f64>().await {
            info!("Scrolled to view {}", res.round());
            VIEW.write().current = View::from_usize(res.round() as usize);
        }
    });

    // Watch for app focus (mobile)
    #[cfg(target_os="android")]
    use_wry_event_handler(|event, window| {
        use dioxus::mobile::tao::event::{Event as WryEvent, WindowEvent};

        use crate::app::controller::{MusicMsg, send_music_msg};

        match event {
            WryEvent::WindowEvent{ event: window_event, window_id: _, .. } => match window_event {
                WindowEvent::Focused(f) => send_music_msg(MusicMsg::UpdateInfo),
                _ => {}
            },
            _ => {},
        }
    });

    use_asset_handler("trackimage", move |request, responder| {
        let r = Response::builder().status(200).body(&[]).unwrap();

        info!("requested track image {:?}, thread {:?}", request.uri(), std::thread::current().id());

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
        document::Stylesheet { href: AUTOPLAYLISTS_CSS }
        document::Stylesheet { href: EXPLORER_CSS }
        document::Stylesheet { href: LIBRARYMANAGEMENT_CSS }
        document::Stylesheet { href: MENUBAR_CSS }
        document::Stylesheet { href: PLAYLISTS_CSS }
        document::Stylesheet { href: SETTINGS_CSS }
        document::Stylesheet { href: TAGEDITOR_CSS }
        document::Stylesheet { href: TRACKVIEW_CSS }
        document::Stylesheet { href: TRACKOPTIONS_CSS }
        document::Stylesheet { href: QUEUE_CSS }

        style {
            r#"
                @font-face {{
                    font-family: Rubik;
                    src: url({asset!("/assets/Rubik/Rubik-VariableFont_wght.ttf")}); 
                }}
            "#
        }

        div {
            class: "loadingpopupbg",
            hidden: loading_track_weights() == tracks_count(),
            div { class: "loadingpopup",
                "Loading weights for track {loading_track_weights} out of {tracks_count} "
            }
        }

        div {
            class: "mainview",
            id: "mainview",
            tabindex: 0,
            autofocus: true,
            padding_top: if cfg!(target_os = "android") { "30pt" },
            background: if cfg!(target_os = "android") && VIEW.read().current != View::Song { "var(--bg)" },

            TrackView { controller }
            QueueList { controller }
            AllTracks { controller }
            AlbumsList { controller }
            ArtistList { controller }
            GenreList { controller }
            PlaylistsView { controller }
            LibraryManagement { controller }
            Settings { controller }

            TrackOptions { controller }
            TagEditorView { controller }

            if DELETING_TRACK.read().is_some() {
                Confirmation {
                    label: "Delete Track {controller.read().all_tracks[DELETING_TRACK().unwrap()].title}?",
                    confirm: move |_| {
                        controller.write().delete_track(&*DB.read(), DELETING_TRACK().unwrap());
                        DELETING_TRACK.set(None);
                    },
                    cancel: |_| {
                        DELETING_TRACK.set(None);
                    },
                }
            }
        }

        MenuBar { controller }
    }
}

#[component]
pub fn MenuBar(controller: SyncSignal<MusicController>) -> Element {
    rsx! {
        div {
            class: "buttonrow nav",
            background_color: if VIEW.read().current != View::Song { "var(--bg)" },
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
            if !controller.read().settings.ui.hide_explorer_buttons {
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
                    background_image: "url({asset!(\"/assets/icons/edit.svg\")})",
                    onclick: move |_| VIEW.write().open(View::LibraryManagement),
                }
            } else {
                button {
                    class: "svg-button",
                    background_image: "url({asset!(\"/assets/icons/playlist.svg\")})",
                    onclick: move |_| VIEW.write().open(View::Playlists),
                }
            }
            button {
                class: "svg-button",
                background_image: "url({asset!(\"/assets/icons/settings.svg\")})",
                onclick: move |_| VIEW.write().open(View::Settings),
            }
        }
    }
}
