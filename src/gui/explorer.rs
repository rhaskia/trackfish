use crate::app::{utils::similar, MusicController};
use crate::{View, VIEW};
use dioxus::prelude::*;

#[component]
pub fn AlbumsList(controller: Signal<MusicController>) -> Element {
    let mut albums = use_signal(|| controller.read().albums.clone());
    let mut is_searching = use_signal(|| false);

    use_future(move || async move {
        albums.write().sort_by(|(_, a), (_, b)| b.cmp(a));
    });

    let mut set_album = move |name| {
        let idx = controller.read().albums.iter().position(|a| a.0 == name).unwrap_or_default();
        VIEW.write().album = Some(idx);
    };

    rsx! {
        div { 
            class: "albums",
            onclick: move |_| is_searching.set(false),
            div {
                class: "searchbar",
                onclick: move |_| is_searching.set(true),
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
                TracksView { controller, viewtype: View::Albums }
            }
        }

    }
}

#[component]
pub fn TracksView(controller: Signal<MusicController>, viewtype: View) -> Element {
    let viewtype = use_signal(|| viewtype);
    let idx = use_memo(move || match viewtype() {
        View::Albums => VIEW.read().album.clone().unwrap(),
        View::Artists => VIEW.read().artist.clone().unwrap(),
        View::Genres => VIEW.read().genre.clone().unwrap(),
        _ => unreachable!(),
    });

    let name = use_signal(|| match viewtype() {
        View::Albums => controller.read().albums[idx()].clone().0,
        View::Artists => controller.read().artists[idx()].clone().0,
        View::Genres => controller.read().genres[idx()].clone().0,
        _ => todo!(),
    });

    let mut tracks = use_signal(move || {
        controller.read().get_tracks_where(|t| match viewtype() {
            View::Albums => similar(&t.album, &name.read()),
            View::Artists => t.has_artist(&name.read()),
            View::Genres => t.has_genre(&name.read()),
            _ => unreachable!(),
        })
    });

    use_future(move || async move {
        if View::Albums == viewtype() {
            tracks.write().sort_by(|a, b| {
                controller.read().all_tracks[*a]
                    .trackno
                    .cmp(&controller.read().all_tracks[*b].trackno)
            });
        }
    });

    rsx! {
        div { class: "tracksviewheader",
            img {
                onclick: move |_| match viewtype() {
                    View::Albums => VIEW.write().album = None,
                    View::Artists => VIEW.write().artist = None,
                    View::Genres => VIEW.write().genre = None,
                    _ => unreachable!(),
                },
                src: "assets/back.svg",
            }
            h3 { "{name()}" }
            img { src: "assets/shuffle.svg" }
        }
        div { class: "tracksview",
            for track in tracks() {
                div {
                    class: "trackitem",
                    onclick: move |_| {
                        match viewtype() {
                            View::Albums => controller.write().play_album_at(name(), track),
                            View::Artists => controller.write().play_artist_at(name(), track),
                            View::Genres => controller.write().play_genre_at(name(), track),
                            _ => unreachable!(),
                        };
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

    let mut set_artist = move |name| {
        let idx = controller.read().artists.iter().position(|a| a.0 == name).unwrap_or_default();
        VIEW.write().artist = Some(idx);
    };

    rsx! {
        div { class: "artists",
            div { class: "searchbar",
                display: if VIEW.read().artist.is_some() { "none" },
                img { src: "assets/search.svg" }
                input {}
            }
            div { id: "artistlist", class: "tracklist",
                display: if VIEW.read().artist.is_some() { "none" },

                for i in 0..artists.read().len() {
                    div { class: "thinitem",
                        onclick: move |_| set_artist(artists.read()[i].clone().0),
                        "{artists.read()[i].0}"
                        br {}
                        small { "{artists.read()[i].1} songs" }
                    }
                }
            }
            if VIEW.read().artist.is_some() {
                TracksView { controller, viewtype: View::Artists }
            }
        }
    }
}

#[component]
pub fn GenreList(controller: Signal<MusicController>) -> Element {
    let mut genres = use_signal(|| controller.read().genres.clone());

    use_future(move || async move {
        genres.write().sort_by(|(_, a), (_, b)| b.cmp(a));
    });

    let mut set_genre = move |name| {
        let idx = controller.read().genres.iter().position(|a| a.0 == name).unwrap_or_default();
        VIEW.write().genre = Some(idx);
    };

    rsx! {
        div { class: "artists",
            div { class: "searchbar",
                display: if VIEW.read().genre.is_some() { "none" },
                img { src: "assets/search.svg" }
                input {}
            }
            div { id: "genrelist", class: "tracklist",
                display: if VIEW.read().genre.is_some() { "none" },
                for i in 0..genres.read().len() {
                    if genres.read()[i].1 > 1 {
                        div {
                            class: "thinitem",
                            onclick: move |_| set_genre(genres.read()[i].0.clone()),
                            "{genres.read()[i].0}",
                            small { "{genres.read()[i].1} songs" }
                        }
                    }
                }
            }
            if VIEW.read().genre.is_some() {
                TracksView { controller, viewtype: View::Genres }
            }
        }
    }
}

