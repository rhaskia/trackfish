use super::{View, TRACKOPTION, VIEW};
use crate::app::{MusicController, Track};
use crate::app::controller::MusicControllerStoreExt;
use crate::gui::icons::*;
use dioxus::prelude::*;
use dioxus::stores::SyncStore;
use log::info;
use std::time::Duration;
use tokio::time;

#[component]
pub fn TrackView(controller: SyncStore<MusicController>) -> Element {
    let mut progress = use_signal(|| controller.progress_secs()());
    let mut progress_held = use_signal(|| false);
    let empty_track = use_signal(Track::default);

    let current_track_idx = move || {
        let current_queue = controller.current_queue()();
        controller.queues().get(current_queue).unwrap().read().current()
    };

    let current_track = move || {
        match controller.all_tracks().get(current_track_idx()) {
            Some(track) => track(),
            None => empty_track(),
        }
    };

    // Skip to next song
    let skip = move |_: Event<MouseData>| {
        controller.write().skip();
        progress.set(0.0);
        info!("{:?}", current_track());
    };

    // Skip to previous song, or start of current song
    let skipback = move |_: Event<MouseData>| {
        controller.write().skipback();
        progress.set(0.0);
        info!("{:?}", current_track());
    };

    // Updates song progress to UI from controller without breaking input slider functionality
    use_future(move || async move {
        loop {
            time::sleep(Duration::from_secs_f64(0.25)).await;
            if !progress_held() && controller.playing()() {
                *controller.progress_secs().write() += 0.25;
                *progress.write() = *controller.progress_secs().read();
            }
        }
    });

    rsx! {
        div { id: "trackview", class: "trackview view",

            // Background image blur
            div {
                class: "trackblur",
                background_image: "url(/trackimage/{current_track_idx()}?origin=trackview)",
            }

            // Main track image
            div { class: "imageview",
                img {
                    src: "/trackimage/{current_track_idx()}?origin=trackview",
                    loading: "onvisible",
                }
            }

            div { class: "trackcontrols",
                h3 { "{current_track().title}" }

                // Song artist list
                span { class: "artistspecifier",
                    for (idx , artist) in current_track().artists.into_iter().enumerate()
                    {
                        // Start each artist with comma after first
                        if idx > 0 {
                            ", "
                        }

                        span {
                            onclick: move |_| {
                                VIEW.write().open(View::Artists);
                                VIEW.write().artist = Some(artist.clone());
                            },
                            "{artist}"
                        }
                    }
                }

                // Song album list
                span {
                    class: "albumspecifier",
                    // Open album view on click
                    onclick: move |_| {
                        VIEW.write().album = Some(
                            current_track().album.to_string(),
                        );
                        VIEW.write().open(View::Albums);
                    },
                    "{current_track().album}"
                }

                // Song genre list
                span { class: "genresspecifier",
                    for genre in current_track().genres.iter().cloned() {
                        span {
                            // Open genre view on click
                            onclick: move |_| {
                                VIEW.write().open(View::Genres);
                                VIEW.write().genre = Some(genre.clone());
                            },
                            "{genre}"
                        }
                    }
                }

                // Track progress information
                div { class: "progressrow",
                    span { class: "songprogress", "{format_seconds(progress())}" }
                    input {
                        r#type: "range",
                        style: "--dist: {progress() / controller.song_length()() * 100.0}%;",
                        value: "{progress}",
                        step: 0.25,
                        max: controller.song_length()(),
                        onchange: move |e| {
                            let value = e.value().parse().unwrap();
                            controller.write().set_pos(value);
                            info!("{:?}", controller.progress_secs().read());
                            progress.set(value)
                        },
                        oninput: move |e| {
                            let value = e.value().parse().unwrap();
                            info!("oninput");
                            progress.set(value);
                        },
                        onmousedown: move |_| progress_held.set(true),
                        onmouseup: move |_| progress_held.set(false),
                    }
                    span { class: "songlength", "{format_seconds(controller.song_length()())}" }
                }

                // Track controls
                div { class: "buttonrow",
                    button {
                        class: "svg-button",
                        background_image: "url({VERT_ICON})",
                        onclick: move |_| *TRACKOPTION.write() = Some(current_track_idx()),
                    }

                    button {
                        class: "svg-button",
                        background_image: "url({SKIP_BACK_ICON})",
                        onclick: skipback,
                    }

                    button {
                        class: "svg-button",
                        background_image: if controller.playing()() { "url({PAUSE_ICON})" } else { "url({PLAY_ICON})" },
                        onclick: move |_| controller.write().toggle_playing(),
                    }

                    button {
                        class: "svg-button",
                        background_image: "url({SKIP_ICON})",
                        onclick: skip,
                    }

                    button {
                        class: "svg-button",
                        background_image: if controller.shuffle()() { "url({SHUFFLE_ON_ICON})" } else { "url({SHUFFLE_ICON})" },
                        onclick: move |_| controller.write().toggle_shuffle(),
                    }
                }
            }
        }
    }
}

fn format_seconds(seconds: f64) -> String {
    let seconds = seconds as i64;
    let s = seconds % 60;
    let minutes = (seconds - s) / 60;
    format!("{minutes:.0}:{s:02.0}")
}