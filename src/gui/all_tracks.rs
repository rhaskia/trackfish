use dioxus::prelude::*;
use crate::app::MusicController;
use crate::app::utils::strip_unnessecary;
use crate::{View, VIEW};
use crate::app::utils::similar;

#[component]
pub fn AllTracks(controller: Signal<MusicController>) -> Element {
    let mut is_searching = use_signal(|| false);
    let tracks = use_signal(|| (0..controller.read().all_tracks.len()).collect::<Vec<usize>>());

    rsx!{
        div {
            class: "searchbar",
            onclick: move |_| is_searching.set(true),
            img { src: "assets/search.svg" }
            div { class: "pseudoinput" }
        }
        div { class: "tracklist",
            for i in 0..controller.read().all_tracks.len() {
                div {
                    class: "trackitem",
                    id: "trackitem-{i}",
                    onclick: move |_| {
                        controller.write().add_all_queue(i);
                        VIEW.write().current = View::Song;
                    },
                    img { src: "/trackimage/{i}" }
                    span { "{controller.read().all_tracks[i].title}" }
                    div { flex_grow: 1 }
                    img { src: "/assets/vert.svg" }
                }
            }
        }
        if is_searching() {
            TracksSearch { controller, tracks, is_searching }
        }
    }
}

#[component]
pub fn TracksSearch(controller: Signal<MusicController>, tracks: Signal<Vec<usize>>, is_searching: Signal<bool>) -> Element {
    let mut search = use_signal(String::new);
    let matches = use_memo(move || {
        let search = strip_unnessecary(&search.read());
        log::info!("{search}");
        if search.is_empty() {
            Vec::new()
        } else {
            tracks.read().iter()
                .filter(|t| strip_unnessecary(&controller.read().all_tracks[**t].title).starts_with(&search))
                .cloned()
                .collect::<Vec<usize>>()
        }
    });

    rsx! {
        div { 
            class: "searchholder",
            onclick: move |_| is_searching.set(false),
            div { flex: 1 }
            div {
                class: "searchpopup",
                div {
                    class: "searchpopupbar",
                    img { src: "assets/search.svg" }
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
                            img { src: "/trackimage/{track}" }
                            span { "{controller.read().all_tracks[track].title}" }
                        }
                    }
                }
            }
            div { flex: 1 }
        }
    }
}
