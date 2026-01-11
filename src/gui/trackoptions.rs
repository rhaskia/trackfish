use crate::app::MusicController;
use crate::gui::EDITING_TAG;

use super::icons::*;
use super::{View, ADD_TO_PLAYLIST, TRACKOPTION, VIEW};
use dioxus::prelude::*;
use super::MOBILE;

#[component]
pub fn TrackOptions(controller: SyncSignal<MusicController>) -> Element {
    let mut drag_amount = use_signal(|| 0.0_f64);
    let mut dragging = use_signal(|| false);
    let mut drag_start = use_signal(|| 0.0);

    let mut track_option = use_signal(|| None);

    use_effect(move || {
        if TRACKOPTION().is_some() {
            track_option.set(TRACKOPTION());
        }
    });

    // Only render if TRACKOPTION is set, eg user intended to open options
    if let Some(track) = track_option() {
        rsx! {
            div {
                class: "optionsbg",
                style: if MOBILE() { "width: 100vw; height: 100vh;" },
                onclick: move |_| {
                    *TRACKOPTION.write() = None;
                    dragging.set(false);
                    drag_amount.set(0.0)
                },
                z_index: if TRACKOPTION.read().is_none() { "-1" },
                display: if !MOBILE() && TRACKOPTION.read().is_none() { "none" },
                div {
                    class: "trackoptions optionbox",
                    class: if MOBILE() { "optionsboxmobile" },
                    style: if MOBILE() { r#"width: 100vw; height: 60vh; left: 0; top: revert; 
                        border: none; border-top: 1px solid #696969; border-radius: 0px; 
                        justify-content: space-between;
                        display: flex; flex-direction: column;"# },
                    onclick: |e| {
                        if MOBILE() {
                            e.stop_propagation()
                        }
                    },
                    ontouchstart: move |e| {
                        dragging.set(true);
                        drag_start.set(e.data().touches()[0].client_coordinates().y)
                    },
                    ontouchend: move |_| {
                        dragging.set(false);
                        if drag_amount() < -200.0 {
                            *TRACKOPTION.write() = None;
                        }
                        drag_amount.set(0.0);
                    },
                    ontouchmove: move |e| {
                        if dragging() {
                            drag_amount.set(drag_start() - e.data().touches()[0].client_coordinates().y);
                        }
                    },
                    bottom: if TRACKOPTION.read().is_none() { "-1000px" } else if dragging() && MOBILE() { "{drag_amount().min(0.0) as i32}px" } else { "0px" },

                    h3 { class: if MOBILE() { "largerheader" },
                        "{controller.read().all_tracks[track].title}"
                    }

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

                    button {
                        onclick: move |_| {
                            controller.write().play_next(track);
                            TRACKOPTION.set(None);
                        },
                        img { src: SKIP_ICON }
                        "Play after this song"
                    }

                    hr {}

                    button {
                        onclick: move |_| {
                            let artist = controller.read().all_tracks[track].artists[0].clone();
                            VIEW.write().open(View::Artists);
                            VIEW.write().artist = Some(artist.clone());
                            TRACKOPTION.set(None);
                        },
                        img { src: ARTIST_ICON }
                        "Go to artist"
                    }

                    button {
                        onclick: move |_| {
                            let album = controller.read().all_tracks[track].album.clone();
                            VIEW.write().open(View::Albums);
                            VIEW.write().album = Some(album.clone());
                            TRACKOPTION.set(None);
                        },
                        img { src: ALBUM_ICON }
                        "Go to album"
                    }

                    hr {}

                    button {
                        onclick: move |_| {
                            let index = TRACKOPTION().unwrap();
                            EDITING_TAG.set(Some((index, controller.read().all_tracks[index].clone())));
                        },
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
