use crate::app::MusicController;

use super::{View, ADD_TO_PLAYLIST, TRACKOPTION, VIEW};
use dioxus::prelude::*;
use super::icons::*;

#[component]
pub fn TrackOptions(controller: SyncSignal<MusicController>) -> Element {
    if let Some(track) = TRACKOPTION() {
        rsx! {
            div {
                class: "optionsbg",
                onclick: move |_| *TRACKOPTION.write() = None,
                div { class: "trackoptions optionbox",
                    h3 { "{controller.read().all_tracks[track].title}" }

                    button {
                        img { src: INFO_ICON }
                        "Track Information"
                    }

                    // View separate options
                    match VIEW.read().current {
                        View::Song => rsx! {
                            TrackOptionsExplorerView { controller, track }
                        },
                        View::Queue => rsx! {
                            TrackOptionsQueueView { controller, track }
                        },
                        View::Playlists => rsx! {
                            TrackOptionsPlaylistsView { controller, track }
                        },
                        _ => rsx! {
                            TrackOptionsExplorerView { controller, track }
                        },
                    }

                    hr {}

                    // Various track options
                    button { onclick: move |_| controller.write().start_radio(track),
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
                    button { onclick: move |_| controller.write().mut_current_queue().cached_order.push(track),
                        img { src: PLAYLIST_PLAY_ICON }
                        "Add to current queue"
                    }
                    button { onclick: move |_| controller.write().play_next(track),
                        img { src: SKIP_ICON }
                        "Play after this song"
                    }
                    hr {}
                    button {
                        onclick: move |_| {
                            let artist = controller.read().all_tracks[track].artists[0].clone();
                            VIEW.write().open(View::Artists);
                            VIEW.write().artist = Some(artist.clone());
                        },
                        img { src: ARTIST_ICON }
                        "Go to artist"
                    }
                    button {
                        onclick: move |_| {
                            let album = controller.read().all_tracks[track].album.clone();
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
pub fn TrackOptionsQueueView(controller: SyncSignal<MusicController>, track: usize) -> Element {
    rsx! {
        button {
            img { src: INFO_ICON }
            "Remove from queue"
        }
    }
}

#[component]
pub fn TrackOptionsTrackView(controller: SyncSignal<MusicController>, track: usize) -> Element {
    rsx! {
        span {}
    }
}

#[component]
pub fn TrackOptionsExplorerView(controller: SyncSignal<MusicController>, track: usize) -> Element {
    rsx! {
        span {}
    }
}

#[component]
pub fn TrackOptionsPlaylistsView(controller: SyncSignal<MusicController>, track: usize) -> Element {
    rsx! {
        button {
            onclick: move |_| {
                controller.write().playlists[VIEW.read().playlist.unwrap()].remove(track);
                controller.write().save_playlist(VIEW.read().playlist.unwrap());
            },
            img { src: REMOVE_ICON }
            "Remove from playlist"
        }
    }
}
