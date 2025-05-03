use dioxus::prelude::*;
use crate::gui::{View, TRACKOPTION, VIEW, CONTROLLER};
use super::TracksView;

#[component]
pub fn ArtistList() -> Element {
    let mut artists = use_signal(|| Vec::new());

    use_effect(move || {
        let mut artists_unsorted = CONTROLLER
            .read()
            .artists
            .clone()
            .into_iter()
            .collect::<Vec<(String, (String, usize))>>();
        artists_unsorted.sort_by(|(_, (_, a)), (_, (_, b))| b.cmp(a));
        artists.set(artists_unsorted);
    });

    let set_artist = move |name| {
        VIEW.write().artist = Some(name);
    };

    rsx! {
        div {
            class: "artists",
            display: if VIEW.read().current != View::Artists { "none" },
            div {
                class: "searchbar",
                display: if VIEW.read().artist.is_some() { "none" },
                img { src: "assets/icons/search.svg" }
                input {}
            }
            div {
                id: "artistlist",
                class: "tracklist",
                display: if VIEW.read().artist.is_some() { "none" },

                for i in 0..artists.read().len() {
                    div {
                        class: "thinitem",
                        onclick: move |_| set_artist(artists.read()[i].clone().1.0),
                        "{artists.read()[i].1.0}"
                        br {}
                        small { "{artists.read()[i].1.1} songs" }
                    }
                }
            }
            if VIEW.read().artist.is_some() {
                TracksView { viewtype: View::Artists }
            }
        }
    }
}
