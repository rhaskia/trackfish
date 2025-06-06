use super::{View, ADD_TO_PLAYLIST, CONTROLLER, TRACKOPTION, VIEW};
use dioxus::prelude::*;

#[component]
pub fn TrackOptions() -> Element {
    if let Some(track) = TRACKOPTION() {
        rsx! {
            div {
                class: "optionsbg",
                onclick: move |_| *TRACKOPTION.write() = None,
                div { class: "trackoptions optionbox",
                    h3 { "{CONTROLLER.read().all_tracks[track].title}" }

                    button {
                        img { src: "assets/icons/info.svg" }
                        "Track Information"
                    }

                    // View separate options
                    match VIEW.read().current {
                        View::Song => rsx! {
                            TrackOptionsExplorerView { track }
                        },
                        View::Queue => rsx! {
                            TrackOptionsQueueView { track }
                        },
                        View::Playlists => rsx! {
                            TrackOptionsPlaylistsView { track }
                        },
                        _ => rsx! {
                            TrackOptionsExplorerView { track }
                        },
                    }

                    hr {}

                    // Various track options
                    button { onclick: move |_| CONTROLLER.write().start_radio(track),
                        img { src: "assets/icons/radio.svg" }
                        "Start radio"
                    }
                    button { onclick: move |_| *ADD_TO_PLAYLIST.write() = Some(track),
                        img { src: "assets/icons/playlistadd.svg" }
                        "Add to a playlist"
                    }
                    button {
                        img { src: "assets/icons/queue.svg" }
                        "Add to a queue"
                    }
                    button { onclick: move |_| CONTROLLER.write().mut_current_queue().cached_order.push(track),
                        img { src: "assets/icons/playlistplay.svg" }
                        "Add to current queue"
                    }
                    button { onclick: move |_| CONTROLLER.write().play_next(track),
                        img { src: "assets/icons/skip.svg" }
                        "Play after this song"
                    }
                    hr {}
                    button {
                        onclick: move |_| {
                            let artist = CONTROLLER.read().all_tracks[track].artists[0].clone();
                            VIEW.write().open(View::Artists);
                            VIEW.write().artist = Some(artist.clone());
                        },
                        img { src: "assets/icons/artist.svg" }
                        "Go to artist"
                    }
                    button {
                        onclick: move |_| {
                            let album = CONTROLLER.read().all_tracks[track].album.clone();
                            VIEW.write().open(View::Albums);
                            VIEW.write().album = Some(album.clone());
                        },
                        img { src: "assets/icons/album.svg" }
                        "Go to album"
                    }
                    hr {}
                    button {
                        img { src: "assets/icons/edit.svg" }
                        "Edit tags"
                    }
                    button {
                        img { src: "assets/icons/delete.svg" }
                        "Delete song from files"
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
pub fn TrackOptionsQueueView(track: usize) -> Element {
    rsx! {
        button {
            img { src: "assets/icons/info.svg" }
            "Remove from queue"
        }
    }
}

#[component]
pub fn TrackOptionsTrackView(track: usize) -> Element {
    rsx! {
        span {}
    }
}

#[component]
pub fn TrackOptionsExplorerView(track: usize) -> Element {
    rsx! {
        span {}
    }
}

#[component]
pub fn TrackOptionsPlaylistsView(track: usize) -> Element {
    rsx! {
        button {
            onclick: move |_| {
                CONTROLLER.write().playlists[VIEW.read().playlist.unwrap()].remove(track);
                CONTROLLER.write().save_playlist(VIEW.read().playlist.unwrap());
            },
            img { src: "assets/icons/remove.svg" }
            "Remove from playlist"
        }
    }
}
