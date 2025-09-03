use super::{View, CONTROLLER, TRACKOPTION, VIEW};
use dioxus::prelude::*;
use log::info;
use std::time::Duration;
use tokio::time;
use crate::app::track::get_track_image;
use crate::gui::icons::*;
use crate::app::controller::controller;

#[component]
pub fn TrackView() -> Element {
    let mut progress = use_signal(|| CONTROLLER.read().progress_secs);
    let mut progress_held = use_signal(|| false);

    let skip = move |_: Event<MouseData>| {
        controller().lock().unwrap().skip();
        progress.set(0.0);
        info!("{:?}", CONTROLLER.read().current_track());
    };

    let skipback = move |_: Event<MouseData>| {
        controller().lock().unwrap().skipback();
        progress.set(0.0);
        info!("{:?}", CONTROLLER.read().current_track());
    };

    use_future(move || async move {
        loop {
            time::sleep(Duration::from_secs_f64(0.25)).await;
            if !progress_held() {
                *progress.write() = CONTROLLER.read().progress_secs;
                // if CONTROLLER.read().track_ended() && CONTROLLER.read().all_tracks.len() > 0
                // {
                //     controller().lock().unwrap().skip();
                // }
            }
        }
    });

    rsx! {
        div {
            class: "trackview",
            display: if VIEW.read().current != View::Song { "none" },

            // Background image blur
            div {
                class: "trackblur",
                // background_image: "url(/trackimage/{CONTROLLER.read().current_track_idx()})",
            }

            // Main track image
            div { class: "imageview",
                img {
                    //src: "/trackimage/{CONTROLLER.read().current_track_idx()}",
                    loading: "onvisible",
                }
            }

            div { class: "trackcontrols",
                h3 { "{CONTROLLER.read().current_track_title().unwrap_or_default()}" }

                // Song artist list
                span { class: "artistspecifier",
                    for (idx , artist) in CONTROLLER
                        .read()
                        .current_track_artist()
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .enumerate()
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
                            CONTROLLER.read().current_track_album().unwrap_or_default().to_string(),
                        );
                        VIEW.write().open(View::Albums);
                    },
                    "{CONTROLLER.read().current_track_album().unwrap_or_default()}"
                }

                // Song genre list
                span { class: "genresspecifier",
                    if let Some(genres) = CONTROLLER.read().current_track_genres() {
                        for genre in genres.iter().cloned() {
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
                }

                // Track progress information
                div { class: "progressrow",
                    span { class: "songprogress",
                        "{format_seconds(CONTROLLER.read().progress_secs)}"
                    }
                    input {
                        r#type: "range",
                        value: progress,
                        step: 0.25,
                        max: CONTROLLER.read().song_length(),
                        onchange: move |e| {
                            let value = e.value().parse().unwrap();
                            controller().lock().unwrap().set_pos(value);
                            progress.set(value)
                        },
                        onmousedown: move |_| progress_held.set(true),
                        onmouseup: move |_| progress_held.set(false),
                    }
                    span { class: "songlength",
                        "{format_seconds(CONTROLLER.read().song_length())}"
                    }
                }

                // Track controls
                div { class: "buttonrow",
                    button {
                        class: "svg-button",
                        background_image: "url({VERT_ICON})",
                        onclick: move |_| *TRACKOPTION.write() = Some(CONTROLLER.read().current_track_idx()),
                    }
                    button {
                        class: "svg-button",
                        background_image: "url({SKIP_BACK_ICON})",
                        onclick: skipback,
                    }
                    button {
                        class: "svg-button",
                        background_image: if CONTROLLER.read().playing() { "url({PAUSE_ICON})" } else { "url({PLAY_ICON})" },
                        onclick: move |_| controller().lock().unwrap().toggle_playing(),
                    }
                    button {
                        class: "svg-button",
                        background_image: "url({SKIP_ICON})",
                        onclick: skip,
                    }
                    button {
                        class: "svg-button",
                        background_image: if CONTROLLER.read().shuffle { "url({SHUFFLE_ON_ICON})" } else { "url({SHUFFLE_ICON})" },
                        onclick: move |_| controller().lock().unwrap().toggle_shuffle(),
                    }
                }
            }
        }
    }
}

fn format_seconds(seconds: f64) -> String {
    let s = seconds % 60.0;
    let minutes = (seconds - s) / 60.0;
    format!("{minutes:.0}:{s:02.0}")
}
