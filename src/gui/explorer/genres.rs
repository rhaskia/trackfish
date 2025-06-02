use dioxus::prelude::*;
use crate::gui::{View, TRACKOPTION, VIEW, CONTROLLER};
use super::TracksView;
use super::TracksSearch;

#[component]
pub fn GenreList() -> Element {
    let mut genres = use_signal(|| Vec::new());
    let mut is_searching = use_signal(|| false);

    use_effect(move || {
        let mut genres_unsorted = CONTROLLER
            .read()
            .genres
            .clone()
            .into_iter()
            .collect::<Vec<(String, usize)>>();
        genres_unsorted.sort_by(|(_, a), (_, b)| b.cmp(a));
        genres.set(genres_unsorted);
    });

    let set_genre = move |name| {
        VIEW.write().genre = Some(name);
    };

    rsx! {
        div {
            class: "artists",
            display: if VIEW.read().current != View::Genres { "none" },
            div { class: "searchbar", onclick: move |_| is_searching.set(true),
                img { src: "assets/icons/search.svg" }
                div { class: "pseudoinput" }
            }
            div {
                id: "genrelist",
                class: "tracklist",
                display: if VIEW.read().genre.is_some() { "none" },
                for i in 0..genres.read().len() {
                    if genres.read()[i].1 > 1 {
                        div {
                            class: "thinitem",
                            id: "genre-{i}",
                            onclick: move |_| set_genre(genres.read()[i].0.clone()),
                            if genres.read()[i].0.is_empty() {
                                "Unknown Genres"
                            } else {
                                "{genres.read()[i].0}"
                            }
                            small { "{genres.read()[i].1} songs" }
                        }
                    }
                }
            }
            if VIEW.read().genre.is_some() {
                TracksView { viewtype: View::Genres }
            }
        }
    }
}

#[component]
pub fn GenreSearch(is_searching: Signal<bool>) -> Element {
    let mut search = use_signal(String::new);
    
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
                    strip_unnessecary(&CONTROLLER.read().genres[**t]).starts_with(&search)
                })
                .cloned()
                .collect::<Vec<usize>>()
        }
    });

    rsx!{
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
                                    &format!("document.getElementById('genre-{}').scrollIntoView();", track),
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
