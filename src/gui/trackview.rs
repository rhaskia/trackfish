use super::{View, CONTROLLER, TRACKOPTION, VIEW};
use dioxus::prelude::*;
use log::info;
use std::time::Duration;
use tokio::time;

#[component]
pub fn TrackView() -> Element {
    let mut progress = use_signal(|| CONTROLLER.read().player.progress_secs());
    let mut progress_held = use_signal(|| false);

    let skip = move |_: Event<MouseData>| {
        CONTROLLER.write().skip();
        progress.set(0.0);
        info!("{:?}", CONTROLLER.read().current_track());
    };

    let skipback = move |_: Event<MouseData>| {
        CONTROLLER.write().skipback();
        progress.set(0.0);
        info!("{:?}", CONTROLLER.read().current_track());
    };

    use_future(move || async move {
        loop {
            time::sleep(Duration::from_secs_f64(0.25)).await;
            if !progress_held() {
                *progress.write() = CONTROLLER.read().player.progress_secs();
                if CONTROLLER.read().player.track_ended() && CONTROLLER.read().all_tracks.len() > 0
                {
                    CONTROLLER.write().skip();
                }
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
                background_image: "url(/trackimage/{CONTROLLER.read().current_track_idx()})",
            }

            // Main track image
            div { class: "imageview",
                img {
                    src: "/trackimage/{CONTROLLER.read().current_track_idx()}",
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
                        "{format_seconds(CONTROLLER.read().player.progress_secs())}"
                    }
                    input {
                        r#type: "range",
                        value: progress,
                        step: 0.25,
                        max: CONTROLLER.read().player.song_length(),
                        onchange: move |e| {
                            let value = e.value().parse().unwrap();
                            CONTROLLER.write().player.set_pos(value);
                            progress.set(value)
                        },
                        onmousedown: move |_| progress_held.set(true),
                        onmouseup: move |_| progress_held.set(false),
                    }
                    span { class: "songlength",
                        "{format_seconds(CONTROLLER.read().player.song_length())}"
                    }
                }

                // Track controls
                div { class: "buttonrow",
                    button {
                        class: "svg-button",
                        background_image: "url(assets/icons/vert.svg)",
                        onclick: move |_| *TRACKOPTION.write() = Some(CONTROLLER.read().current_track_idx()),
                    }
                    button {
                        class: "svg-button",
                        background_image: "url(assets/icons/skipprevious.svg)",
                        onclick: skipback,
                    }
                    button {
                        class: "svg-button",
                        background_image: "url(assets/icons/pause.svg)",
                        //background_image: if CONTROLLER.read().playing() { "url(assets/icons/pause.svg)" } else { "url(assets/icons/play.svg)" },
                        onclick: move |_| info!("what the hell"),
                    }
                    button {
                        class: "svg-button",
                        background_image: "url(assets/icons/skip.svg)",
                        onclick: skip,
                    }
                    button {
                        class: "svg-button",
                        background_image: if CONTROLLER.read().shuffle { "url(assets/icons/shuffleon.svg)" } else { "url(assets/icons/shuffle.svg)" },
                        onclick: move |_| CONTROLLER.write().toggle_shuffle(),
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
