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

use crate::document::eval;

#[cfg(not(target_os = "android"))]
use dioxus::desktop::use_asset_handler;
#[cfg(target_os = "android")]
use dioxus::mobile::use_asset_handler;

use app::{track::load_tracks, MusicController};
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

#[cfg(not(target_os = "android"))]
fn init() {
    use dioxus::mobile::tao::window::Icon;

    LogTracer::init().expect("Failed to initialize LogTracer");

    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).unwrap();

    let png = &include_bytes!("../assets/icons/icon256.png")[..];
    let header = minipng::decode_png_header(png).expect("bad PNG");
    let mut buffer = vec![0; header.required_bytes_rgba8bpc()];
    let mut image = minipng::decode_png(png, &mut buffer).expect("bad PNG");
    image.convert_to_rgba8bpc();
    let pixels = image.pixels();
    let icon = Icon::from_rgba(pixels.to_vec(), image.width(), image.height()).unwrap();

    let window = WindowBuilder::new()
        .with_title("TrackFish")
        .with_always_on_top(false)
        .with_window_icon(Some(icon));
    let config = dioxus::desktop::Config::new().with_window(window);
    LaunchBuilder::new().with_cfg(config).launch(SetUpRoute);
}

#[component]
fn SetUpRoute() -> Element {
    use trackfish::app::settings::Settings;
    let set_up = use_signal(Settings::exists);

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
                onchange: |e| info!("{e:?}"),
            }
            // Other options
            br {}
            br {}
            button { "Confirm" }
        }
    }
}

#[component]
fn App() -> Element {
    let mut controller = use_signal(|| MusicController::empty());

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
        #[cfg(target_os = "android")]
        {
            let result = crossbow::Permission::StorageRead.request_async().await;
            info!("{result:?}");
        }

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

    // spawn(async {
    //     #[cfg(target_os = "android")]
    //     {
    //         use jni::JNIEnv;
    //         use jni::objects::JClass;
    //         use jni::sys::jint;
    //         use jni::objects::JValue;
    //
    //         let ctx = ndk_context::android_context();
    //         let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    //         let mut env = vm.attach_current_thread().unwrap();
    //         let class_ctx = env.find_class("android/content/Context").unwrap();
    //
    //         let media_session_class = env.find_class("android/media/session/MediaSession").unwrap();
    //         let tag = env.new_string("MyMediaSession").expect("Failed to create Java string");
    //
    //         let context = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };
    //
    //         let media_session = env.new_object(
    //             media_session_class,
    //             "(Landroid/content/Context;Ljava/lang/String;)V",
    //             &[JValue::Object(&context), JValue::Object(&tag.into())],
    //         ).expect("Failed to create MediaSession object");
    //
    //         info!("hi!");
    //
    //         let flag_handles_media_buttons = 1; // MediaSession.FLAG_HANDLES_MEDIA_BUTTONS
    //         let flag_handles_transport_controls = 2; // MediaSession.FLAG_HANDLES_TRANSPORT_CONTROLS
    //         let flags = flag_handles_media_buttons | flag_handles_transport_controls;
    //         env.call_method(
    //             &media_session,
    //             "setFlags",
    //             "(I)V",
    //             &[JValue::Int(flags)],
    //         ).unwrap();
    //
    //         // Set the session active
    //         env.call_method(
    //             &media_session,
    //             "setActive",
    //             "(Z)V",
    //             &[JValue::Bool(1)],
    //         ).unwrap();
    //
    //         info!("hi!");
    //     }
    // });

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
            // AllTracks { controller }
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
