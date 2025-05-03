use dioxus::prelude::*;
use crate::gui::{View, TRACKOPTION, VIEW, CONTROLLER};
use super::TracksView;

#[component]
pub fn GenreList() -> Element {
    let mut genres = use_signal(|| Vec::new());

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
            div {
                class: "searchbar",
                display: if VIEW.read().genre.is_some() { "none" },
                img { src: "assets/icons/search.svg" }
                input {}
            }
            div {
                id: "genrelist",
                class: "tracklist",
                display: if VIEW.read().genre.is_some() { "none" },
                for i in 0..genres.read().len() {
                    if genres.read()[i].1 > 1 {
                        div {
                            class: "thinitem",
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
