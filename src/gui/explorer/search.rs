use dioxus::prelude::*;
use crate::gui::{View, TRACKOPTION, VIEW, CONTROLLER};
use super::TracksView;
use crate::app::utils::strip_unnessecary;

#[component]
pub fn SearchView() -> Element {
    let mut search = use_signal(String::new);
    let clean_search = use_memo(move || strip_unnessecary(&search.read()));
    
    let tracks = use_memo(move || {
        if clean_search().is_empty() {
            Vec::new()
        } else {
            (0..CONTROLLER.read().all_tracks.len())
                .filter(|t| {
                    strip_unnessecary(&CONTROLLER.read().all_tracks[*t].title).starts_with(&clean_search())
                })
                .collect::<Vec<usize>>()
        }
    });

    let artists = use_memo(move || {
        if clean_search().is_empty() {
            Vec::new()
        } else {
            CONTROLLER.read().artists
                .iter()
                .map(|t| t.1.0.clone())
                .filter(|t| {
                    strip_unnessecary(&t).starts_with(&clean_search())
                })
                .collect::<Vec<String>>()
        }
    });

    let albums = use_memo(move || {
        if clean_search().is_empty() {
            Vec::new()
        } else {
            CONTROLLER
                .read()
                .albums
                .iter()
                .map(|a| a.0)
                .filter(|t| {
                    strip_unnessecary(&t).starts_with(&clean_search())
                })
                .cloned()
                .collect::<Vec<String>>()
        }
    });

    let genres = use_memo(move || {
        if clean_search().is_empty() {
            Vec::new()
        } else {
            CONTROLLER
                .read()
                .genres
                .iter()
                .map(|t| t.0.clone())
                .filter(|t| {
                    strip_unnessecary(&t).starts_with(&clean_search())
                })
                .collect::<Vec<String>>()
        }
    });

    rsx!{
        div {
            class: "searchview",
            height: "calc(100vh - 50px)",
            overflow: "hidden",
            display: if VIEW.read().current != View::Search { "none" },
            div {
                class: "searchbar",
                img { src: "assets/icons/search.svg" }
                input {
                    oninput: move |e| search.set(e.value()),
                    value: search,
                }
            }
            div {
                class: "searchviewresults",
                h3 { display: if tracks.read().len() == 0 { "none" }, "{tracks.read().len()} track/s" }
                for track in tracks() {
                    div {
                        class: "trackitem",
                        img { class: "trackitemicon", src: "/trackimage/{track}" }
                        span { "{CONTROLLER.read().all_tracks[track].title}" }
                    }
                }
                h3 { display: if tracks.read().len() == 0 { "none" }, "{tracks.read().len()} album/s" }
                for album in albums() {
                    div {
                        class: "trackitem",
                        img { class: "trackitemicon", src: "/trackimage/{CONTROLLER.read().get_album_artwork(album.clone())}" }
                        span { "{album}" }
                    }
                }
                h3 { display: if artists.read().len() == 0 { "none" }, "{artists.read().len()} artist/s" }
                for artist in artists() {
                    div {
                        class: "thinitem",
                        "{artist}"
                    }
                }
                h3 { display: if genres.read().len() == 0 { "none" }, "{artists.read().len()} genre/s" }
                for genre in genres() {
                    div {
                        class: "thinitem",
                        "{genre}"
                    }
                }

            }
        }
    }
}

#[component]
pub fn TracksSearch(tracks: Memo<Vec<usize>>, is_searching: Signal<bool>, id_prefix: String) -> Element {
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
                    strip_unnessecary(&CONTROLLER.read().all_tracks[**t].title).starts_with(&search)
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
                    img { src: "assets/icons/search.svg" }
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
                                    &format!("document.getElementById('{id_prefix}-trackitem-{}').scrollIntoView();", track),
                                );
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
