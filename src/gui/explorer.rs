use dioxus::prelude::*;
use crate::app::QueueManager;

#[component]
pub fn AlbumsList(queue: Signal<QueueManager>) -> Element {
    rsx! {
        div {
            id: "albumlist",
            class: "tracklist",
            div {
                class: "searchbar",
                img { src: "assets/search.svg" }
                input {}
            },
            for (artist, songs) in &queue.read().artists {
                div {
                    class: "thinitem",
                    "{artist}"
                }
            }
        }
    }
}

#[component]
pub fn ArtistList(queue: Signal<QueueManager>) -> Element {
    rsx! {
        div {
            id: "artistlist",
            class: "tracklist",
            div {
                class: "searchbar",
                img { src: "assets/search.svg" }
                input {}
            },
            for (artist, songs) in &queue.read().artists {
                div {
                    class: "thinitem",
                    "{artist}"
                }
            }
        }
    }
}

#[component]
pub fn GenreList(queue: Signal<QueueManager>) -> Element {
    rsx! {
        div {
            id: "genrelist",
            class: "tracklist",
            for (genre, freq) in &queue.read().genres {
                if *freq > 1 {
                    div {
                        class: "thinitem",
                        "{genre}"
                    }
                }
            }
        }
    }
}
