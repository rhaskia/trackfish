use crate::app::MusicController;
use crate::app::utils::strip_unnessecary;
use crate::gui::{View, VIEW, icons::*};
use dioxus::prelude::*;

#[component]
pub fn SearchView(controller: SyncSignal<MusicController>) -> Element {
    let mut search = use_signal(String::new);
    let clean_search = use_memo(move || strip_unnessecary(&search.read()));

    let tracks = use_memo(move || {
        if clean_search().is_empty() {
            Vec::new()
        } else {
            (0..controller.read().all_tracks.len())
                .filter(|t| {
                    strip_unnessecary(&controller.read().all_tracks[*t].title)
                        .starts_with(&clean_search())
                })
                .collect::<Vec<usize>>()
        }
    });

    let artists = use_memo(move || {
        if clean_search().is_empty() {
            Vec::new()
        } else {
            controller
                .read()
                .artists
                .iter()
                .map(|t| t.1 .0.clone())
                .filter(|t| strip_unnessecary(&t).starts_with(&clean_search()))
                .collect::<Vec<String>>()
        }
    });

    let albums = use_memo(move || {
        if clean_search().is_empty() {
            Vec::new()
        } else {
            controller
                .read()
                .albums
                .iter()
                .map(|a| a.0)
                .filter(|t| strip_unnessecary(&t).starts_with(&clean_search()))
                .cloned()
                .collect::<Vec<String>>()
        }
    });

    let genres = use_memo(move || {
        if clean_search().is_empty() {
            Vec::new()
        } else {
            controller
                .read()
                .genres
                .iter()
                .map(|t| t.0.clone())
                .filter(|t| strip_unnessecary(&t).starts_with(&clean_search()))
                .collect::<Vec<String>>()
        }
    });

    rsx! {
        div {
            class: "searchview",
            height: "calc(100vh - 50px)",
            overflow: "hidden",
            display: if VIEW.read().current != View::Search { "none" },
            div { class: "searchbar",
                img { src: SEARCH_ICON }
                input { oninput: move |e| search.set(e.value()), value: search }
            }
            div { class: "searchviewresults",
                h3 { display: if tracks.read().len() == 0 { "none" }, "{tracks.read().len()} track/s" }
                for i in 0..tracks.read().len() {
                    div {
                        class: "trackitem",
                        onclick: move |_| {
                            controller.write().add_all_queue(tracks.read()[i]);
                            VIEW.write().current = View::Song;
                        },
                        img {
                            class: "trackitemicon",
                            src: "/trackimage/{tracks.read()[i]}",
                        }
                        span { "{controller.read().all_tracks[tracks.read()[i]].title}" }
                    }
                }
                h3 { display: if albums.read().len() == 0 { "none" }, "{albums.read().len()} album/s" }
                for i in 0..albums.len() {
                    div {
                        class: "trackitem",
                        onclick: move |_| {
                            VIEW.write().open(View::Albums);
                            VIEW.write().album = Some(albums.read()[i].clone());
                        },
                        img {
                            class: "trackitemicon",
                            src: "/trackimage/{controller.read().get_album_artwork(albums.read()[i].clone())}",
                        }
                        span { "{albums.read()[i]}" }
                    }
                }
                h3 { display: if artists.read().len() == 0 { "none" }, "{artists.read().len()} artist/s" }
                for i in 0..artists.read().len() {
                    div {
                        class: "thinitem",
                        onclick: move |_| {
                            VIEW.write().open(View::Artists);
                            VIEW.write().artist = Some(artists.read()[i].clone());
                        },
                        "{artists.read()[i]}"
                    }
                }
                h3 { display: if genres.read().len() == 0 { "none" }, "{genres.read().len()} genre/s" }
                for i in 0..genres.read().len() {
                    div {
                        class: "thinitem",
                        onclick: move |_| {
                            VIEW.write().open(View::Genres);
                            VIEW.write().genre = Some(genres.read()[i].clone());
                        },
                        "{genres.read()[i]}"
                    }
                }
            
            }
        }
    }
}

#[component]
pub fn TracksSearch(
    controller: SyncSignal<MusicController>,
    tracks: Memo<Vec<usize>>,
    is_searching: Signal<bool>,
    id_prefix: String,
) -> Element {
    let mut search = use_signal(String::new);
    let id_prefix = use_signal(|| id_prefix);

    let matches = use_memo(move || {
        let search = strip_unnessecary(&search.read());
        log::info!("searching {search}");

        if search.is_empty() {
            Vec::new()
        } else {
            tracks
                .read()
                .iter()
                .filter(|t| {
                    strip_unnessecary(&controller.read().all_tracks[**t].title).starts_with(&search)
                })
                .cloned()
                .collect::<Vec<usize>>()
        }
    });

    rsx! {
        div { class: "searchholder", onclick: move |_| is_searching.set(false),
            div { flex: 1 }
            div { class: "searchpopup",
                div { class: "searchpopupbar",
                    img { src: SEARCH_ICON }
                    input {
                        value: search,
                        autofocus: true,
                        onclick: |e| e.stop_propagation(),
                        oninput: move |e| search.set(e.value()),
                    }
                }
                div { class: "searchtracks",
                    for track in matches() {
                        div {
                            class: "trackitem",
                            onclick: move |_| {
                                document::eval(
                                    &format!(
                                        "document.getElementById('{id_prefix}-trackitem-{}').scrollIntoView();",
                                        track,
                                    ),
                                );
                            },
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
