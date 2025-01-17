use dioxus::prelude::*;
use crate::app::{MusicController, utils::similar};

#[component]
pub fn AlbumsList(controller: Signal<MusicController>) -> Element {
    let mut selected = use_signal(|| None);
    let mut albums = use_signal(|| controller.read().albums.clone());

    use_future(move || async move {
        albums.write().sort_by(|(_, a), (_, b)| b.cmp(a));
    });

    let mut set_album = move |name| {
        let idx = controller.read().albums.iter().position(|a| a.0 == name).unwrap_or_default();
        selected.set(Some(idx));
    };
    
    rsx! {
        div {
            class: "albums",
            div {
                class: "searchbar",
                img { src: "assets/search.svg" }
                input {}
            },
            div {
                id: "albumlist",
                class: "tracklist",
                display: if selected().is_some() { "none" }, 

                for i in 0..albums.read().len() {
                    div {
                        class: "thinitem",
                        onclick: move |_| set_album(albums.read()[i].0.clone()),
                        span { "{albums.read()[i].0}" }
                        br {} 
                        small {
                            "{albums.read()[i].1} songs"
                        }
                    }
                }
            }
            if let Some(idx) = selected() {
                AlbumView { controller, idx }
            }
        }
    }
}

#[component]
pub fn AlbumView(controller: Signal<MusicController>, idx: usize) -> Element {
    let album = controller.read().albums[idx].clone().0;
    let tracks = use_memo(move || controller.read().get_tracks_where(|t| similar(&t.album, &album)));

    rsx!{
        for track in tracks() {
            div {
                class: "trackitem",
                img { src: "/trackimage/{track}" }
                span { "{controller.read().get_track(track).unwrap().title}" }
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
