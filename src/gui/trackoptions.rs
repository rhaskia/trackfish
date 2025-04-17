use dioxus::prelude::*;
use super::{View, VIEW, ADD_TO_PLAYLIST, CONTROLLER};

#[component]
pub fn TrackOptions() -> Element {
    if let Some(track) = super::TRACKOPTION() {
        rsx!{
            div {
                class: "trackoptionsbg", 
                onclick: move |_| super::TRACKOPTION.set(None),
                div {
                    class: "trackoptions",
                    h3 { "{CONTROLLER.read().all_tracks[track].title}" }

                    button {
                        img { src: "assets/icons/info.svg" }
                        "Track Information"
                    }

                    // View separate options
                    match VIEW.read().current {
                        View::Song => rsx!{ TrackOptionsExplorerView { track } },
                        View::Queue => rsx!{ TrackOptionsQueueView { track } },
                        View::Playlists => rsx!{ TrackOptionsPlaylistsView { track } },
                        _ => rsx!{ TrackOptionsExplorerView { track } }
                    }

                    hr {}

                    // Various track options
                    button {
                        onclick: move |_| CONTROLLER.write().start_radio(track),
                        img { src: "assets/icons/radio.svg" }
                        "Start radio"
                    }
                    button {
                        onclick: move |_| ADD_TO_PLAYLIST.set(Some(track)),
                        img { src: "assets/icons/playlistadd.svg" }
                        "Add to a playlist"
                    }
                    button {
                        img { src: "assets/icons/queue.svg" }
                        "Add to a queue"
                    }
                    button {
                        onclick: move |_| CONTROLLER.write().mut_current_queue().cached_order.push(track),
                        img { src: "assets/icons/playlistplay.svg" }
                        "Add to current queue"
                    }
                    button {
                        onclick: move |_| CONTROLLER.write().play_next(track),
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
                    hr { }
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
        rsx!{}
    }
}

#[component]
pub fn TrackOptionsQueueView(track: usize) -> Element {
    rsx!{
        button {
            img { src: "assets/icons/info.svg" }
            "Remove from queue"
        }
    }
}

#[component]
pub fn TrackOptionsTrackView(track: usize) -> Element {
    rsx!{
        span {}
    }
}

#[component]
pub fn TrackOptionsExplorerView(track: usize) -> Element {
    rsx!{
        span {}
    }
}

#[component]
pub fn TrackOptionsPlaylistsView(track: usize) -> Element {
    rsx!{
        button {
            img { src: "assets/icons/info.svg" }
            "Remove from playlist"
        }
    }
}
