use super::{View, TRACKOPTION, VIEW};
use crate::app::MusicController;
use crate::gui::icons::*;
use dioxus_native::prelude::*;
use log::info;
use std::time::Duration;
use tokio::time;
use ndarray::Array1;

#[component]
pub fn TrackView(controller: SyncSignal<MusicController>) -> Element {
    let mut progress = use_signal(|| controller.read().progress_secs);
    let mut progress_held = use_signal(|| false);

    // Skip to next song
    let skip = move |_: Event<MouseData>| {
        controller.write().skip();
        progress.set(0.0);
        info!("{:?}", controller.read().current_track());
    };

    let track_info = use_memo(move || {
        controller.read().track_info.get(controller.read().current_track_idx()).cloned().unwrap_or_default()
    });

    // Skip to previous song, or start of current song
    let skipback = move |_: Event<MouseData>| {
        controller.write().skipback();
        progress.set(0.0);
        info!("{:?}", controller.read().current_track());
    };

    // Updates song progress to UI from controller without breaking input slider functionality
    use_future(move || async move {
        loop {
            time::sleep(Duration::from_secs_f64(0.25)).await;
            if !progress_held() && controller.read().playing() {
                controller.write().progress_secs += 0.25;
                *progress.write() = controller.read().progress_secs;
            }
        }
    });

    rsx! {
        div { id: "trackview", class: "trackview view",

            // Background image blur
            div {
                class: "trackblur",
                background_image: "url(/trackimage/{controller.read().current_track_idx()}?origin=trackview)",
            }

            // Main track image
            div { class: "imageview",
                img {
                    src: "/trackimage/{controller.read().current_track_idx()}?origin=trackview",
                    loading: "onvisible",
                }
            }

            div { class: "trackcontrols",
                h3 { "{controller.read().current_track_title().unwrap_or_default()}" }

                // Song artist list
                span { class: "artistspecifier",
                    for (idx , artist) in controller
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
                            controller.read().current_track_album().unwrap_or_default().to_string(),
                        );
                        VIEW.write().open(View::Albums);
                    },
                    "{controller.read().current_track_album().unwrap_or_default()}"
                }

                // Song genre list
                span { class: "genresspecifier",
                    if let Some(genres) = controller.read().current_track_genres() {
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
                    span { class: "songprogress", "{format_seconds(progress())}" }
                    input {
                        r#type: "range",
                        style: "--dist: {progress() / controller.read().song_length * 100.0}%;",
                        value: "{progress}",
                        step: 0.25,
                        max: controller.read().song_length,
                        onchange: move |e| {
                            let value = e.value().parse().unwrap();
                            controller.write().set_pos(value);
                            info!("{:?}", controller.read().progress_secs);
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
                    span { class: "songlength", "{format_seconds(controller.read().song_length)}" }
                }

                // Track controls
                div { class: "buttonrow",
                    button {
                        class: "svg-button",
                        background_image: "url({VERT_ICON})",
                        onclick: move |_| *TRACKOPTION.write() = Some(controller.read().current_track_idx()),
                    }

                    button {
                        class: "svg-button",
                        background_image: "url({SKIP_BACK_ICON})",
                        onclick: skipback,
                    }

                    button {
                        class: "svg-button",
                        background_image: if controller.read().playing() { "url({PAUSE_ICON})" } else { "url({PLAY_ICON})" },
                        onclick: move |_| controller.write().toggle_playing(),
                    }

                    button {
                        class: "svg-button",
                        background_image: "url({SKIP_ICON})",
                        onclick: skip,
                    }

                    button {
                        class: "svg-button",
                        background_image: if controller.read().shuffle { "url({SHUFFLE_ON_ICON})" } else { "url({SHUFFLE_ICON})" },
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