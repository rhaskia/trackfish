use dioxus::prelude::*;
use crate::queue::QueueManager;
use std::time::Duration;
use tokio::time;
use log::info;

#[component]
pub fn TrackView(queue: Signal<QueueManager>) -> Element {
    let mut progress = use_signal(|| queue.read().player.progress_secs());
    let mut progress_held = use_signal(|| false);

    let skip = move |e: Event<MouseData>| {
        queue.write().skip();
        progress.set(0.0);
        info!("{:?}", queue.read().current_track());
    };

    let skipback = move |e: Event<MouseData>| {
        queue.write().skipback();
        progress.set(0.0);
        info!("{:?}", queue.read().current_track());
    };
    
    use_future(move || async move {
        loop {
            time::sleep(Duration::from_secs_f64(0.25)).await;
            if !progress_held() {
                *progress.write() = queue.read().player.progress_secs();
                if queue.read().player.track_ended() {
                    queue.write().skip();
                }
            }
        }
    });

    rsx! {
        div {
            class: "songview",
            div {
                class: "imageview",
                img {
                    src: "/trackimage/{queue.read().current()}"
                }
            }
            h3 {
                "{queue.read().current_track_title().unwrap_or_default()}"
            }
            span { 
                class: "artistspecifier",
                for (idx, artist) in queue.read().current_track_artist().cloned().unwrap_or_default().into_iter().enumerate() {
                    if idx > 0 {
                        " & "
                    }
                    span { 
                        onclick: move |_| queue.write().add_artist_queue(artist.to_string()),
                        "{artist}"
                    },
                }
            }
            span { 
                class: "albumspecifier",
                onclick: move |e| queue.write().add_current_album_queue(),
                "{queue.read().current_track_album().unwrap_or_default()}" 
            }
            span {
                class: "genresspecifier",
                if let Some(genres) = queue.read().current_track_genres() {
                    for genre in genres {
                        span {
                            "{genre}"
                        }
                    }
                }
            }
            div {
                class: "progressrow",
                span {
                    class: "songprogress",
                    "{format_seconds(queue.read().player.progress_secs())}"
                }
                input {
                    r#type: "range",
                    value: progress,
                    step: 0.25,
                    max: queue.read().player.song_length(),
                    onchange: move |e| {
                        let value = e.value().parse().unwrap();
                        queue.write().player.set_pos(value);
                        progress.set(value)
                    },
                    onmousedown: move |e| progress_held.set(true),
                    onmouseup: move |e| progress_held.set(false),
                }
                span {
                    class: "songlength",
                    "{format_seconds(queue.read().player.song_length())}"
                }
            }
            div {
                class: "buttonrow",
                button {
                    class: "like-button",
                    class: "svg-button",
                }
                button {
                    class: "skipprev-button",
                    class: "svg-button",
                    onclick: skipback,
                }
                button {
                    class: "svg-button",
                    onclick: move |e| queue.write().toggle_playing(),
                    background_image: if queue.read().playing() { "url(assets/pause.svg)" } else { "url(assets/play.svg)" },
                }
                button {
                    class: "skip-button",
                    class: "svg-button",
                    onclick: skip,
                }
                button {
                    class: "dislike-button",
                    class: "svg-button",
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
