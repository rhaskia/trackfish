use dioxus::prelude::*;
use crate::app::MusicController;
use std::time::Duration;
use tokio::time;
use log::info;
use super::{View, VIEW};
use super::input::{key_to_action, Action};

#[component]
pub fn TrackView(controller: Signal<MusicController>) -> Element {
    let mut progress = use_signal(|| controller.read().player.progress_secs());
    let mut progress_held = use_signal(|| false);

    let skip = move |_: Event<MouseData>| {
        controller.write().skip();
        progress.set(0.0);
        info!("{:?}", controller.read().current_track());
    };

    let skipback = move |_: Event<MouseData>| {
        controller.write().skipback();
        progress.set(0.0);
        info!("{:?}", controller.read().current_track());
    };
    
    use_future(move || async move {
        loop {
            time::sleep(Duration::from_secs_f64(0.25)).await;
            if !progress_held() {
                *progress.write() = controller.read().player.progress_secs();
                if controller.read().player.track_ended() && controller.read().all_tracks.len() > 0 {
                    controller.write().skip();
                }
            }
        }
    });

    rsx! {
        div { class: "trackview",
            onkeydown: move |e| match key_to_action(e) {
                Some(Action::Skip) => controller.write().skip(),
                Some(Action::SkipPrevious) => controller.write().skipback(),
                Some(Action::PauseToggle) => controller.write().toggle_playing(),
                _ => {},
            },
            display: if VIEW.read().current != View::Song { "none" },
            div { class: "trackblur",
                background_image: "url(/trackimage/{controller.read().current_track_idx()})" 
            }
            div { class: "imageview",
                img { src: "/trackimage/{controller.read().current_track_idx()}" }
            }
            div {
                class: "trackoptions",
                h3 { "{controller.read().current_track_title().unwrap_or_default()}" }
                span { class: "artistspecifier",
                    for (idx , artist) in controller
                        .read()
                        .current_track_artist()
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .enumerate()
                    {
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
                span {
                    class: "albumspecifier",
                    onclick: move |_| {
                        VIEW.write().album = Some(controller.read().current_track_album().unwrap_or_default().to_string());
                        VIEW.write().open(View::Albums);
                    },
                    // TODO: open Album View for current album
                    "{controller.read().current_track_album().unwrap_or_default()}"
                }
                span { class: "genresspecifier",
                    if let Some(genres) = controller.read().current_track_genres() {
                        for genre in genres.iter().cloned() {
                            span { 
                                onclick: move |_| {
                                    VIEW.write().open(View::Genres);
                                    VIEW.write().genre = Some(genre.clone()); 
                                },
                                "{genre}"
                            }
                        }
                    }
                }
                div { class: "progressrow",
                    span { class: "songprogress",
                        "{format_seconds(controller.read().player.progress_secs())}"
                    }
                    input {
                        r#type: "range",
                        value: progress,
                        step: 0.25,
                        max: controller.read().player.song_length(),
                        onchange: move |e| {
                            let value = e.value().parse().unwrap();
                            controller.write().player.set_pos(value);
                            progress.set(value)
                        },
                        onmousedown: move |_| progress_held.set(true),
                        onmouseup: move |_| progress_held.set(false),
                    }
                    span { class: "songlength", "{format_seconds(controller.read().player.song_length())}" }
                }
                div { class: "buttonrow",
                    button { class: "like-button", class: "svg-button" }
                    button {
                        class: "skipprev-button",
                        class: "svg-button",
                        onclick: skipback,
                    }
                    button {
                        class: "svg-button playpause",
                        onclick: move |_| controller.write().toggle_playing(),
                        class: if controller.read().playing() { "pause" },
                    }
                    button {
                        class: "skip-button",
                        class: "svg-button",
                        onclick: skip,
                    }
                    button {
                        class: "shuffle-button",
                        class: "svg-button",
                        class: if controller.read().shuffle { "shuffle-on" },
                        onclick: move |_| controller.write().toggle_shuffle(),
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
