use super::CONTROLLER;
use super::{View, VIEW};
use crate::app::{utils::similar, MusicController};
use rand::Rng;
use dioxus::prelude::*;

#[component]
pub fn TracksView(viewtype: View) -> Element {
    let viewtype = use_signal(|| viewtype);
    let mut explorer_settings = use_signal(|| false);
    let mut adding_to_playlist = use_signal(|| false);
    let mut adding_to_queue = use_signal(|| false);

    let name = use_memo(move || match viewtype() {
        View::Albums => VIEW.read().album.clone().unwrap(),
        View::Artists => VIEW.read().artist.clone().unwrap(),
        View::Genres => VIEW.read().genre.clone().unwrap(),
        View::Playlists => CONTROLLER.read().playlists[VIEW.read().playlist.unwrap()].name.clone(),
        _ => unreachable!(),
    });

    let tracks = use_memo(move || {
        if let View::Playlists = viewtype() {
            return CONTROLLER.read().playlists[VIEW.read().playlist.unwrap()].tracks.clone();
        }

        let mut tracks = CONTROLLER.read().get_tracks_where(|t| match viewtype() {
            View::Albums => similar(&t.album, &name.read()),
            View::Artists => t.has_artist(&name.read()),
            View::Genres => t.has_genre(&name.read()),
            _ => unreachable!(),
        });

        if viewtype() == View::Albums {
            tracks.sort_by(|a, b| {
                CONTROLLER.read().all_tracks[*a]
                    .trackno
                    .cmp(&CONTROLLER.read().all_tracks[*b].trackno)
            });
        }

        tracks
    });

    rsx! {
        div { class: "tracksviewheader",
            img {
                onclick: move |_| match viewtype() {
                    View::Albums => VIEW.write().album = None,
                    View::Artists => VIEW.write().artist = None,
                    View::Genres => VIEW.write().genre = None,
                    View::Playlists => VIEW.write().playlist = None,
                    _ => unreachable!(),
                },
                src: "assets/icons/back.svg",
            }
            h3 {
                if name().is_empty() {
                    "Unknown {viewtype():?}"
                } else { "{name()}" }
            }
            img {
                onclick: move |_| explorer_settings.set(true),
                src: "assets/icons/vert.svg"
            }
        }
        div { class: "tracksview",
            for track in tracks() {
                div {
                    class: "trackitem",
                    onclick: move |_| {
                        match viewtype() {
                            View::Albums => CONTROLLER.write().play_album_at(name(), track),
                            View::Artists => CONTROLLER.write().play_artist_at(name(), track),
                            View::Genres => CONTROLLER.write().play_genre_at(name(), track),
                            View::Playlists => CONTROLLER.write().start_playlist_at(VIEW.read().playlist.unwrap(), track),
                            _ => unreachable!(),
                        };
                        VIEW.write().open(View::Song);
                    },
                    img { class: "trackitemicon", src: "/trackimage/{track}", loading: "onvisible" }
                    span { "{CONTROLLER.read().get_track(track).unwrap().title}" }
                    div { flex_grow: 1 }
                    img {
                        class: "trackbutton",
                        loading: "onvisible",
                        onclick: move |e| {
                            e.stop_propagation();
                            super::TRACKOPTION.set(Some(track));
                        },
                        src: "/assets/icons/vert.svg"
                    }
                }
            }
        }

        if explorer_settings() {
            ExplorerOptions { explorer_settings, name, viewtype, tracks, adding_to_queue, adding_to_playlist }
        }

        if adding_to_playlist() {
            div {
                class: "optionsbg",
                onclick: move |_| adding_to_playlist.set(false),
                div {
                    class: "playlistadder",
                    h3 { "Add {name()} to a playlist" }

                    for i in 0..CONTROLLER.read().playlists.len() {
                        button {
                            onclick: move |_| {
                                CONTROLLER.write().add_tracks_to_playlist(i, tracks());
                                adding_to_playlist.set(false);
                            },
                            "{CONTROLLER.read().playlists[i].name}"
                        }
                    }
                }
            }
        }

        if adding_to_queue() {
            div {
                class: "optionsbg",
                onclick: move |_| adding_to_queue.set(false),
                div {
                    class: "playlistadder",
                    h3 { "Add {name()} to a queue" }

                    for i in 0..CONTROLLER.read().queues.len() {
                        button {
                            onclick: move |_| {
                                CONTROLLER.write().add_tracks_to_queue(i, tracks());
                                adding_to_queue.set(false);
                            },
                            "{CONTROLLER.read().queues[i].queue_type}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AlbumsList() -> Element {
    let mut albums = use_signal(|| Vec::new());
    let mut is_searching = use_signal(|| false);

    use_effect(move || {
        let mut albums_unsorted =
            CONTROLLER.read().albums.clone().into_iter().collect::<Vec<(String, usize)>>();
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
                        if albums.read()[i].0.is_empty() {
                            span { "Unknown Album" }
                        } else {
                            span { "{albums.read()[i].0}" }
                        }
                        br {}
                        small { "{albums.read()[i].1} songs" }
                    }
                }
            }
            if VIEW.read().album.is_some() {
                TracksView { viewtype: View::Albums }
            }
        }

    }
}



#[component]
pub fn ExplorerOptions(
    explorer_settings: Signal<bool>,
    adding_to_playlist: Signal<bool>,
    adding_to_queue: Signal<bool>,
    name: Memo<String>,
    viewtype: Signal<View>,
    tracks: Memo<Vec<usize>>,
) -> Element {
    rsx! {
        div {
            class: "optionsbg",
            onclick: move |_| explorer_settings.set(false),
            div {
                class: "optionbox",
                style: "--width: 300px; --height: 160px;",
                h3 { "{name}" }
                button {
                    onclick: move |_| {
                        match viewtype() {
                            View::Albums => CONTROLLER.write().play_album_at(name(), tracks.read()[0]),
                            View::Artists => CONTROLLER.write().play_artist_at(name(), tracks.read()[0]),
                            View::Genres => CONTROLLER.write().play_genre_at(name(), tracks.read()[0]),
                            View::Playlists => CONTROLLER.write().start_playlist_at(VIEW.read().playlist.unwrap(), tracks.read()[0]),
                            _ => unreachable!(),
                        };
                        VIEW.write().open(View::Song);
                    },
                    img { src: "assets/icons/play.svg" }
                    "Play"
                }
                button {
                    onclick: move |_| {
                        let random_index = rand::thread_rng().gen_range(0..tracks.read().len());
                        let track = tracks.read()[random_index];
                        match viewtype() {
                            View::Albums => CONTROLLER.write().play_album_at(name(), track),
                            View::Artists => CONTROLLER.write().play_artist_at(name(), track),
                            View::Genres => CONTROLLER.write().play_genre_at(name(), track),
                            View::Playlists => CONTROLLER.write().start_playlist_at(VIEW.read().playlist.unwrap(), track),
                            _ => unreachable!(),
                        };
                        VIEW.write().open(View::Song);
                        CONTROLLER.write().toggle_shuffle();

                        if !CONTROLLER.read().shuffle {
                            CONTROLLER.write().toggle_shuffle();
                        }
                    },
                    img { src: "assets/icons/shuffle.svg" }
                    "Shuffle"
                }
                button {
                    onclick: move |_| adding_to_playlist.set(true),
                    img { src: "assets/icons/playlistadd.svg" }
                    "Add to a playlist"
                }
                button {
                    onclick: move |_| adding_to_queue.set(true),
                    img { src: "assets/icons/queue.svg" }
                    "Add to a queue"
                }
            }
        }
    }
}

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

#[component]
pub fn GenreList() -> Element {
    let mut genres = use_signal(|| Vec::new());

    use_effect(move || {
        let mut genres_unsorted =
            CONTROLLER.read().genres.clone().into_iter().collect::<Vec<(String, usize)>>();
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
                TracksView { viewtype: View::Genres }
            }
        }
    }
}
