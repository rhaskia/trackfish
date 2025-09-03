use super::{View, ADD_TO_PLAYLIST, CONTROLLER, TRACKOPTION, VIEW};
use dioxus::prelude::*;
use super::icons::*;
use crate::app::controller::controller;

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
                        img { src: INFO_ICON }
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
                    button { onclick: move |_| controller().lock().unwrap().start_radio(track),
                        img { src: RADIO_ICON }
                        "Start radio"
                    }
                    button { onclick: move |_| *ADD_TO_PLAYLIST.write() = Some(track),
                        img { src: PLAYLIST_ADD_ICON }
                        "Add to a playlist"
                    }
                    button {
                        img { src: QUEUE_ICON }
                        "Add to a queue"
                    }
                    button { onclick: move |_| controller().lock().unwrap().mut_current_queue().cached_order.push(track),
                        img { src: PLAYLIST_PLAY_ICON }
                        "Add to current queue"
                    }
                    button { onclick: move |_| controller().lock().unwrap().play_next(track),
                        img { src: SKIP_ICON }
                        "Play after this song"
                    }
                    hr {}
                    button {
                        onclick: move |_| {
                            let artist = CONTROLLER.read().all_tracks[track].artists[0].clone();
                            VIEW.write().open(View::Artists);
                            VIEW.write().artist = Some(artist.clone());
                        },
                        img { src: ARTIST_ICON }
                        "Go to artist"
                    }
                    button {
                        onclick: move |_| {
                            let album = CONTROLLER.read().all_tracks[track].album.clone();
                            VIEW.write().open(View::Albums);
                            VIEW.write().album = Some(album.clone());
                        },
                        img { src: ALBUM_ICON }
                        "Go to album"
                    }
                    hr {}
                    button {
                        img { src: EDIT_ICON }
                        "Edit tags"
                    }
                    button {
                        img { src: DELETE_ICON }
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
            img { src: INFO_ICON }
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
                controller().lock().unwrap().playlists[VIEW.read().playlist.unwrap()].remove(track);
                controller().lock().unwrap().save_playlist(VIEW.read().playlist.unwrap());
            },
            img { src: REMOVE_ICON }
            "Remove from playlist"
        }
    }
}
