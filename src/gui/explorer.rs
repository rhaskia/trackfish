use dioxus::prelude::*;
use crate::app::{MusicController, utils::similar};
use crate::{View, VIEW};

#[component]
pub fn AlbumsList(controller: Signal<MusicController>) -> Element {
    let mut albums = use_signal(|| controller.read().albums.clone());

    use_future(move || async move {
        albums.write().sort_by(|(_, a), (_, b)| b.cmp(a));
    });

    let mut set_album = move |name| {
        let idx = controller.read().albums.iter().position(|a| a.0 == name).unwrap_or_default();
        VIEW.write().album = Some(idx);
    };
    
    rsx! {
        div { class: "albums",
            div {
                class: "searchbar",
                display: if VIEW.read().album.is_some() { "none" },
                img { src: "assets/search.svg" }
                input {}
            }
            div {
                id: "albumlist",
                class: "tracklist",
                display: if VIEW.read().album.is_some() { "none" },

                for i in 0..albums.read().len() {
                    div {
                        class: "thinitem",
                        onclick: move |_| set_album(albums.read()[i].0.clone()),
                        span { "{albums.read()[i].0}" }
                        br {}
                        small { "{albums.read()[i].1} songs" }
                    }
                }
            }
            if VIEW.read().album.is_some() {
                AlbumView { controller }
            }
        }
    }
}

#[component]
pub fn AlbumView(controller: Signal<MusicController>) -> Element {
    let idx = use_memo(|| VIEW.read().album.clone().unwrap());
    let album = use_signal(|| controller.read().albums[idx()].clone().0);
    let mut tracks = use_signal(move || controller.read().get_tracks_where(|t| similar(&t.album, &album.read())));

    use_future(move || async move {
        tracks.write().sort_by(|a, b| 
            controller.read().all_tracks[*a].trackno.cmp(&controller.read().all_tracks[*b].trackno));  
    });

    rsx!{
        div { class: "albumviewheader",
            img {
                onclick: move |_| VIEW.write().album = None,
                src: "assets/back.svg",
            }
            h3 { "{album()}" }
            img { src: "assets/shuffle.svg" }
        }
        div { class: "albumview",
            for track in tracks() {
                div {
                    class: "trackitem",
                    onclick: move |_| {
                        controller.write().play_album_at(album(), track);
                        VIEW.write().open(View::Song);
                    },
                    img { src: "/trackimage/{track}" }
                    span { "{controller.read().get_track(track).unwrap().title}" }
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
        div { class: "artists",
            div { class: "searchbar",
                img { src: "assets/search.svg" }
                input {}
            }
            div { id: "artistlist", class: "tracklist",

                for (artist , songs) in &*artists.read() {
                    div { class: "thinitem",
                        "{artist}"
                        br {}
                        small { "{songs} songs" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn GenreList(controller: Signal<MusicController>) -> Element {
    rsx! {
        div { id: "genrelist", class: "tracklist",
            for (genre , freq) in &controller.read().genres {
                if *freq > 1 {
                    div { class: "thinitem", "{genre}" }
                }
            }
        }
    }
}
