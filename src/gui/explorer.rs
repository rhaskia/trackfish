use dioxus::prelude::*;
use crate::app::MusicController;

#[component]
pub fn AlbumsList(controller: Signal<MusicController>) -> Element {
    rsx! {
        div {
            id: "albumlist",
            class: "tracklist",
            div {
                class: "searchbar",
                img { src: "assets/search.svg" }
                input {}
            },
            for (artist, songs) in &controller.read().artists {
                div {
                    class: "thinitem",
                    "{artist}"
                }
            }
        }
    }
}

#[component]
pub fn ArtistList(controller: Signal<MusicController>) -> Element {
    let mut artists = use_signal(|| controller.read().artists.clone());

    use_future(move || async move {
        artists.write().sort_by(|(_, a), (_, b)| b.cmp(a));
    });

    rsx! {
        div {
            class: "artists",
            div {
                class: "searchbar",
                img { src: "assets/search.svg" }
                input {}
            },
            div {
                id: "artistlist",
                class: "tracklist",

                for (artist, songs) in &*artists.read() {
                    div {
                        class: "thinitem",
                        "{artist}"
                        br {}
                        small {
                            "{songs} songs"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn GenreList(controller: Signal<MusicController>) -> Element {
    rsx! {
        div {
            id: "genrelist",
            class: "tracklist",
            for (genre, freq) in &controller.read().genres {
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
