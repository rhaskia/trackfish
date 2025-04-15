use dioxus::prelude::*;
use crate::app::MusicController;
use crate::app::utils::strip_unnessecary;
use super::{View, VIEW};

fn display_time(total: u64) -> String {
    let seconds = total % 60;
    let minutes = (total % 3600 - seconds) / 60;
    let hours = (total - minutes) / 3600;

    format!("{hours}:{minutes:02}:{seconds:02}")
}

use super::CONTROLLER;

#[component]
pub fn AllTracks() -> Element {
    let mut is_searching = use_signal(|| false);
    let tracks = use_memo(move || (0..CONTROLLER.read().all_tracks.len()).collect::<Vec<usize>>());
    let total_time = use_memo(move || CONTROLLER.read().all_tracks.iter().map(|t| t.len).sum::<f64>() as u64);

    rsx!{
        div {
            class: "alltracksview",
            display: if VIEW.read().current != View::AllTracks { "none" },
            div {
                class: "searchbar",
                onclick: move |_| is_searching.set(true),
                img { src: "assets/icons/search.svg" }
                div { class: "pseudoinput" }
            }
            div {
                color: "white",
                padding: "10px",
                "{CONTROLLER.read().all_tracks.len()} songs / "
                "{display_time(total_time())} total duration"
            }
            div { class: "tracklist",
                // extremely slow to load all tracks
                for i in 0..CONTROLLER.read().all_tracks.len().min(100) {
                    div {
                        class: "trackitem",
                        id: "trackitem-{i}",
                        onclick: move |_| {
                            CONTROLLER.write().add_all_queue(i);
                            VIEW.write().current = View::Song;
                        },
                        img { class: "trackitemicon", loading: "onvisible", src: "/trackimage/{i}" }
                        span { "{CONTROLLER.read().all_tracks[i].title}" }
                        div { flex_grow: 1 }
                        img { 
                            class: "trackbutton",
                            loading: "onvisible",
                            src: "/assets/icons/vert.svg"
                        }
                    }
                }
            }
            if is_searching() {
                TracksSearch { tracks, is_searching }
            }
        }
    }
}

#[component]
pub fn TracksSearch(tracks: Memo<Vec<usize>>, is_searching: Signal<bool>) -> Element {
    let mut search = use_signal(String::new);
    let matches = use_memo(move || {
        let search = strip_unnessecary(&search.read());
        log::info!("searching {search}");
        if search.is_empty() {
            log::info!("searching {search}");
            Vec::new()
        } else {
            tracks.read().iter()
                .filter(|t| strip_unnessecary(&CONTROLLER.read().all_tracks[**t].title).starts_with(&search))
                .cloned()
                .collect::<Vec<usize>>()
        }
    });

    rsx! {
        div { 
            class: "searchholder",
            onclick: move |_| is_searching.set(false),
            div { flex: 1 }
            div { "{matches:?}" }
            div {
                class: "searchpopup",
                div {
                    class: "searchpopupbar",
                    img { src: "assets/icons/search.svg" }
                    input {
                        value: search,
                        autofocus: true,
                        onclick: |e| e.stop_propagation(),
                        oninput: move |e| search.set(e.value()),
                    }
                }
                div {
                    class: "searchtracks",
                    for track in matches() {
                        div {
                            class: "trackitem",
                            onclick: move |_| { 
                                document::eval(&format!(
                                    "document.getElementById('trackitem-{}').scrollIntoView();",
                                    track
                                ));
                            },
                            img { src: "/trackimage/{track}" }
                            span { "{CONTROLLER.read().all_tracks[track].title}" }
                        }
                    }
                }
            }
            div { flex: 1 }
        }
    }
}
