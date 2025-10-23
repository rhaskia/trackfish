pub mod albums;
pub mod alltracks;
pub mod artists;
pub mod genres;
pub mod search;

pub use albums::AlbumsList;
pub use alltracks::AllTracks;
pub use artists::ArtistList;
pub use genres::GenreList;
pub use search::{SearchView, TracksSearch};

use super::{View, TRACKOPTION, VIEW};
use crate::app::utils::similar;
use crate::app::MusicController;
use dioxus::document::eval;
use dioxus::prelude::*;
use log::info;
use rand::Rng;

use super::icons::*;

#[component]
pub fn ExplorerSwitch(controller: SyncSignal<MusicController>) -> Element {
    rsx!{
        div {
            class: "explorerswitch",
            button {
                onclick: move |_| VIEW.write().current = View::Albums,
                class: if VIEW.read().current == View::Albums { "explorerselected" },
                "Albums"
            }
            button {
                onclick: move |_| VIEW.write().current = View::Artists,
                class: if VIEW.read().current == View::Artists { "explorerselected" },
                "Artists"
            }
            button {
                onclick: move |_| VIEW.write().current = View::Genres,
                class: if VIEW.read().current == View::Genres { "explorerselected" },
                "Genres"
            }
            button {
                onclick: move |_| VIEW.write().current = View::Playlists,
                class: if VIEW.read().current == View::Playlists { "explorerselected" },
                "Playlists"
            }
        }
    }
}

#[component]
pub fn TracksView(controller: SyncSignal<MusicController>, viewtype: View) -> Element {
    let viewtype = use_signal(|| viewtype);
    let mut explorer_settings = use_signal(|| false);
    let mut adding_to_playlist = use_signal(|| false);
    let mut adding_to_queue = use_signal(|| false);

    let mut window_size = use_signal(|| 0);
    const ROW_HEIGHT: usize = 62;
    const BUFFER_ROWS: usize = 5;

    // Memo to hold the view name for any given viewtype
    let name = use_memo(move || match viewtype() {
        View::Albums => VIEW.read().album.clone().unwrap(),
        View::Artists => VIEW.read().artist.clone().unwrap(),
        View::Genres => VIEW.read().genre.clone().unwrap(),
        View::Playlists => controller.read().playlists[VIEW.read().playlist.unwrap()]
            .name
            .clone(),
        _ => unreachable!(),
    });

    // Tracks to show dependant on viewtype
    let tracks = use_memo(move || {
        if let View::Playlists = viewtype() {
            return controller.read().playlists[VIEW.read().playlist.unwrap()]
                .tracks
                .clone();
        }

        let mut tracks = controller.read().get_tracks_where(|t| match viewtype() {
            View::Albums => similar(&t.album, &name.read()),
            View::Artists => t.has_artist(&name.read()),
            View::Genres => t.has_genre(&name.read()),
            _ => unreachable!(),
        });

        if viewtype() == View::Albums {
            tracks.sort_by(|a, b| {
                controller.read().all_tracks[*a]
                    .trackno
                    .cmp(&controller.read().all_tracks[*b].trackno)
            });
        }

        tracks
    });

    // Virtualization management
    // Start index is where the list is rendered from
    let mut start_index = use_signal(|| 0);
    // Rows in view is the rendered number of items
    let mut rows_in_view = use_signal(|| 15);
    // End index is where rendering stops
    let end_index = use_memo(move || (start_index() + rows_in_view()).min(tracks.read().len()));

    // Watches the list height
    use_future(move || async move {
        let mut js = eval(&format!(
            r#"
            new ResizeObserver(() => {{
                let container = document.getElementById("tracksview-{0}");
                dioxus.send(container.offsetHeight);
            }}).observe(document.getElementById("tracksview-{0}"));
        "#,
            name()
        ));

        loop {
            let height = js.recv::<usize>().await;
            if let Ok(height) = height {
                if height == 0 {
                    continue;
                } // Stops app freezing on opening a different view
                window_size.set(height);
                rows_in_view.set((height / ROW_HEIGHT) + BUFFER_ROWS);
                info!("Window Height {height}");
                info!("ROWS: {}", window_size() / ROW_HEIGHT);
            }
        }
    });

    // Watches the current scroll amount in the list
    use_effect(move || {
        let mut js = eval(&format!(
            r#"
            let container = document.getElementById('tracksview-{0}');
            container.addEventListener('scroll', function(event) {{
                dioxus.send(container.scrollTop);
            }});
            "#,
            name()
        ));

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
        // View header
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

            img { onclick: move |_| explorer_settings.set(true), src: VERT_ICON }
        }

        // Track view list
        div {
            class: "tracksview",
            id: "tracksview-{name()}",
            position: "relative",

            // Allows infinite scroll without having to render every track item before and after
            // the current viewport
            div { min_height: "{(tracks.read().len()) * ROW_HEIGHT}px" }

            // Only render what's needed
            for i in start_index()..end_index() {
                div {
                    class: "trackitem",
                    position: "absolute",
                    style: "top: {i * ROW_HEIGHT}px; position: absolute;",
                    onclick: move |_| {
                        match viewtype() {
                            View::Albums => controller.write().play_album_at(name(), tracks.read()[i]),
                            View::Artists => controller.write().play_artist_at(name(), tracks.read()[i]),
                            View::Genres => controller.write().play_genre_at(name(), tracks.read()[i]),
                            View::Playlists => {
                                controller
                                    .write()
                                    .play_playlist_at(VIEW.read().playlist.unwrap(), tracks.read()[i])
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

                    span { "{controller.read().get_track(tracks.read()[i]).unwrap().title}" }

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
                controller,
                explorer_settings,
                name,
                viewtype,
                tracks,
                adding_to_queue,
                adding_to_playlist,
            }
        }

        // Adding all tracks from current view to a playlist 
        if adding_to_playlist() {
            div {
                class: "optionsbg",
                onclick: move |_| adding_to_playlist.set(false),
                div { class: "playlistadder",
                    h3 { "Add {name()} to a playlist" }

                    for i in 0..controller.read().playlists.len() {
                        button {
                            onclick: move |_| {
                                controller.write().add_tracks_to_playlist(i, tracks());
                                adding_to_playlist.set(false);
                            },
                            "{controller.read().playlists[i].name}"
                        }
                    }
                }
            }
        }

        // Adding all tracks from current view to a queue 
        if adding_to_queue() {
            div {
                class: "optionsbg",
                onclick: move |_| adding_to_queue.set(false),
                div { class: "playlistadder",
                    h3 { "Add {name()} to a queue" }

                    for i in 0..controller.read().queues.len() {
                        button {
                            onclick: move |_| {
                                controller.write().add_tracks_to_queue(i, tracks());
                                adding_to_queue.set(false);
                            },
                            "{controller.read().queues[i].queue_type}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ExplorerOptions(
    controller: SyncSignal<MusicController>,
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
                            View::Albums => controller.write().play_album_at(name(), tracks.read()[0]),
                            View::Artists => controller.write().play_artist_at(name(), tracks.read()[0]),
                            View::Genres => controller.write().play_genre_at(name(), tracks.read()[0]),
                            View::Playlists => {
                                controller
                                    .write()
                                    .play_playlist_at(VIEW.read().playlist.unwrap(), tracks.read()[0])
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
                            View::Albums => controller.write().play_album_at(name(), track),
                            View::Artists => controller.write().play_artist_at(name(), track),
                            View::Genres => controller.write().play_genre_at(name(), track),
                            View::Playlists => {
                                controller
                                    .write()
                                    .play_playlist_at(VIEW.read().playlist.unwrap(), track)
                            }
                            _ => unreachable!(),
                        };
                        VIEW.write().open(View::Song);
                        controller.write().toggle_shuffle();
                        if !controller.read().shuffle {
                            controller.write().toggle_shuffle();
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
