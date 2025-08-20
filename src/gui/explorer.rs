pub mod albums;
pub mod artists;
pub mod alltracks;
pub mod genres;
pub mod search;

pub use albums::AlbumsList;
pub use artists::ArtistList;
pub use alltracks::AllTracks;
pub use genres::GenreList;
pub use search::{SearchView, TracksSearch};

use super::CONTROLLER;
use super::{View, TRACKOPTION, VIEW};
use crate::app::utils::similar;
use dioxus::prelude::*;
use log::info;
use dioxus::document::eval;
use rand::Rng;

use super::icons::*;

#[component]
pub fn TracksView(viewtype: View) -> Element {
    let viewtype = use_signal(|| viewtype);
    let mut explorer_settings = use_signal(|| false);
    let mut adding_to_playlist = use_signal(|| false);
    let mut adding_to_queue = use_signal(|| false);

    let mut window_size = use_signal(|| 0);
    const ROW_HEIGHT: usize = 62;
    const BUFFER_ROWS: usize = 5;

    let name = use_memo(move || match viewtype() {
        View::Albums => VIEW.read().album.clone().unwrap(),
        View::Artists => VIEW.read().artist.clone().unwrap(),
        View::Genres => VIEW.read().genre.clone().unwrap(),
        View::Playlists => CONTROLLER.read().playlists[VIEW.read().playlist.unwrap()]
            .name
            .clone(),
        _ => unreachable!(),
    });

    let tracks = use_memo(move || {
        if let View::Playlists = viewtype() {
            return CONTROLLER.read().playlists[VIEW.read().playlist.unwrap()]
                .tracks
                .clone();
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

    let mut start_index = use_signal(|| 0);
    let rows_in_view = use_memo(move || window_size() / ROW_HEIGHT + BUFFER_ROWS);
    let end_index = use_memo(move || (start_index() + rows_in_view()).min(tracks.read().len()));

    use_future(move || async move {
        let mut js = eval(
            &format!(r#"
            new ResizeObserver(() => {{
                let container = document.getElementById("tracksview-{0}");
                dioxus.send(container.offsetHeight);
            }}).observe(document.getElementById("tracksview-{0}"));
        "#, name()),
        );

        loop {
            let height = js.recv::<usize>().await;
            if let Ok(height) = height {
                if height == 0 { continue; } // Stops app freezing on opening a different view 
                window_size.set(height);
                info!("Window Height {height}");
            }
        }
    });

    use_effect(move || {
        let mut js = eval(
            &format!(r#"
            let container = document.getElementById('tracksview-{0}');
            container.addEventListener('scroll', function(event) {{
                dioxus.send(container.scrollTop);
            }});
            "#, name()),
        );

        spawn(async move {
            loop {
                let scroll_top = js.recv::<usize>().await;
                if let Ok(scroll_top) = scroll_top {
                    let new_index = (scroll_top as f32 / ROW_HEIGHT as f32).floor() as usize;
                    if new_index != start_index() {
                        info!("{new_index}");
                        start_index.set(new_index);
                    }
                }
            }
        });
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
                src: BACK_ICON,
            }
            h3 {
                if name().is_empty() {
                    "Unknown {viewtype():?}"
                } else {
                    "{name()}"
                }
            }
            img {
                onclick: move |_| explorer_settings.set(true),
                src: VERT_ICON,
            }
        }
        div {
            class: "tracksview",
            id: "tracksview-{name()}",
            position: "relative",

            div { min_height: "{(tracks.read().len()) * ROW_HEIGHT}px" }

            for i in start_index()..end_index() {
                div {
                    class: "trackitem",
                    position: "absolute",
                    style: "top: {i * ROW_HEIGHT}px; position: absolute;",
                    onclick: move |_| {
                        match viewtype() {
                            View::Albums => CONTROLLER.write().play_album_at(name(), tracks.read()[i]),
                            View::Artists => CONTROLLER.write().play_artist_at(name(), tracks.read()[i]),
                            View::Genres => CONTROLLER.write().play_genre_at(name(), tracks.read()[i]),
                            View::Playlists => {
                                CONTROLLER
                                    .write()
                                    .start_playlist_at(VIEW.read().playlist.unwrap(), tracks.read()[i])
                            }
                            _ => unreachable!(),
                        };
                        VIEW.write().open(View::Song);
                    },
                    img {
                        class: "trackitemicon",
                        src: "/trackimage/{tracks.read()[i]}",
                        loading: "onvisible",
                    }
                    span { "{CONTROLLER.read().get_track(tracks.read()[i]).unwrap().title}" }
                    div { flex_grow: 1 }
                    img {
                        class: "trackbutton",
                        loading: "onvisible",
                        onclick: move |e| {
                            e.stop_propagation();
                            *TRACKOPTION.write() = Some(tracks.read()[i]);
                        },
                        src: VERT_ICON,
                    }
                }
            }
        }

        if explorer_settings() {
            ExplorerOptions {
                explorer_settings,
                name,
                viewtype,
                tracks,
                adding_to_queue,
                adding_to_playlist,
            }
        }

        if adding_to_playlist() {
            div {
                class: "optionsbg",
                onclick: move |_| adding_to_playlist.set(false),
                div { class: "playlistadder",
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
                div { class: "playlistadder",
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
            div { class: "optionbox", style: "--width: 300px; --height: 160px;",
                h3 { "{name}" }
                button {
                    onclick: move |_| {
                        match viewtype() {
                            View::Albums => CONTROLLER.write().play_album_at(name(), tracks.read()[0]),
                            View::Artists => CONTROLLER.write().play_artist_at(name(), tracks.read()[0]),
                            View::Genres => CONTROLLER.write().play_genre_at(name(), tracks.read()[0]),
                            View::Playlists => {
                                CONTROLLER
                                    .write()
                                    .start_playlist_at(VIEW.read().playlist.unwrap(), tracks.read()[0])
                            }
                            _ => unreachable!(),
                        };
                        VIEW.write().open(View::Song);
                    },
                    img { src: PLAY_ICON }
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
                            View::Playlists => {
                                CONTROLLER
                                    .write()
                                    .start_playlist_at(VIEW.read().playlist.unwrap(), track)
                            }
                            _ => unreachable!(),
                        };
                        VIEW.write().open(View::Song);
                        CONTROLLER.write().toggle_shuffle();
                        if !CONTROLLER.read().shuffle {
                            CONTROLLER.write().toggle_shuffle();
                        }
                    },
                    img { src: SHUFFLE_ICON }
                    "Shuffle"
                }
                button { onclick: move |_| adding_to_playlist.set(true),
                    img { src: PLAYLIST_ADD_ICON }
                    "Add to a playlist"
                }
                button { onclick: move |_| adding_to_queue.set(true),
                    img { src: QUEUE_ICON }
                    "Add to a queue"
                }
            }
        }
    }
}

