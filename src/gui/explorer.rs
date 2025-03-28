use crate::app::{utils::similar, MusicController};
use super::{View, VIEW};
use dioxus::prelude::*;

#[component]
pub fn AlbumsList(controller: Signal<MusicController>) -> Element {
    let mut albums = use_signal(|| Vec::new());
    let mut is_searching = use_signal(|| false);

    use_effect(move || {
        let mut albums_unsorted = controller.read().albums.clone().into_iter().collect::<Vec<(String, usize)>>();
        albums_unsorted.sort_by(|(_, a), (_, b)| b.cmp(a));
        albums.set(albums_unsorted);
    });

    let set_album = move |name| {
        VIEW.write().album = Some(name);
    };

    rsx! {
        div { 
            class: "albums",
            display: if VIEW.read().current != View::Albums { "none" },
            autofocus: true,
            onkeydown: move |e| log::info!("{e:?}"),
            onclick: move |_| is_searching.set(false),
            div {
                class: "searchbar",
                onclick: move |_| is_searching.set(true),
                display: if VIEW.read().album.is_some() { "none" },
                img { src: "assets/icons/search.svg" }
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
    let name = use_memo(move || match viewtype() {
        View::Albums => VIEW.read().album.clone().unwrap(),
        View::Artists => VIEW.read().artist.clone().unwrap(),
        View::Genres => VIEW.read().genre.clone().unwrap(),
        _ => unreachable!(),
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
                src: "assets/icons/back.svg",
            }
            h3 { 
                if name().is_empty() {
                    "Unknown {viewtype():?}"
                } else { "{name()}" }
            }
            img { src: "assets/icons/shuffle.svg" }
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
    let mut artists = use_signal(|| Vec::new());

    use_effect(move || {
        let mut artists_unsorted = controller.read().artists.clone().into_iter().collect::<Vec<(String, usize)>>();
        artists_unsorted.sort_by(|(_, a), (_, b)| b.cmp(a));
        artists.set(artists_unsorted);
    });

    let set_artist = move |name| {
        VIEW.write().artist = Some(name);
    };

    rsx! {
        div { class: "artists",
            display: if VIEW.read().current != View::Artists { "none" },
            div { class: "searchbar",
                display: if VIEW.read().artist.is_some() { "none" },
                img { src: "assets/icons/search.svg" }
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
    let mut genres = use_signal(|| Vec::new());

    use_effect(move || {
        let mut genres_unsorted = controller.read().genres.clone().into_iter().collect::<Vec<(String, usize)>>();
        genres_unsorted.sort_by(|(_, a), (_, b)| b.cmp(a));
        genres.set(genres_unsorted);
    });

    let set_genre = move |name| {
        VIEW.write().genre = Some(name);
    };

    rsx! {
        div { class: "artists",
            display: if VIEW.read().current != View::Genres { "none" },
            div { class: "searchbar",
                display: if VIEW.read().genre.is_some() { "none" },
                img { src: "assets/icons/search.svg" }
                input {}
            }
            div { id: "genrelist", class: "tracklist",
                display: if VIEW.read().genre.is_some() { "none" },
                for i in 0..genres.read().len() {
                    if genres.read()[i].1 > 1 {
                        div {
                            class: "thinitem",
                            onclick: move |_| set_genre(genres.read()[i].0.clone()),
                            if genres.read()[i].0.is_empty() {
                                "Unknown Genres"
                            } else {
                                "{genres.read()[i].0}",
                            }
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

