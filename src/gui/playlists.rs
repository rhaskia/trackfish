use dioxus::prelude::*;
use crate::app::playlist::Playlist;
use super::{CONTROLLER, VIEW, View, ADD_TO_PLAYLIST};
use super::explorer::TracksView;
    
const CREATING_PLAYLIST: GlobalSignal<bool> = Signal::global(|| false);

#[component]
pub fn PlaylistsView() -> Element {
    let mut playlist_name = use_signal(String::new);

    rsx!{
        div {
            class: "playlistsview",
            hidden: VIEW.read().current != View::Playlists,
            div {
                padding: "10px",
                hidden: VIEW.read().playlist.is_some(),
                h3 {
                    "Playlists"
                }
                hr {}

                // Playlist list
                for i in 0..CONTROLLER.read().playlists.len() {
                    div {
                        class: "playlistitem",
                        onclick: move |_| VIEW.write().playlist = Some(i),
                        img { src: "assets/icons/playlistplay.svg" }
                        "{CONTROLLER.read().playlists[i].name}",
                        div { flex: "1 1 0" },
                        img { src: "assets/icons/vert.svg" }
                    }
                }

                button { 
                    onclick: move |_| CREATING_PLAYLIST.set(true),
                    "Create new playlist"
                }

                // Player creation menu
                div {
                    class: "playlistcreatorbg",
                    hidden: !CREATING_PLAYLIST(),
                    onclick: move |_| CREATING_PLAYLIST.set(false),
                    div {
                        class: "playlistcreator",
                        onclick: |e| e.stop_propagation(),
                        label { "Playlist Name:" }
                        input {
                            onchange: move |e| playlist_name.set(e.data().value()), 
                            r#type: "text",
                            value: "{playlist_name}",
                        }
                        button {
                            onclick: move |_| {
                                CONTROLLER.write().playlists.push(Playlist::new(playlist_name()));
                                CREATING_PLAYLIST.set(false);
                                playlist_name.set(String::new());
                            },
                            disabled: playlist_name.read().is_empty(),
                            "Create"
                        }
                    }
                }
            }

            if VIEW.read().playlist.is_some() {
                TracksView { viewtype: View::Playlists }
            }
        }

        if ADD_TO_PLAYLIST.read().is_some() {
            PlaylistAdder {}
        }
    }
}

#[component]
pub fn PlaylistAdder() -> Element {
    rsx! {
        div {
            class: "playlistadderbg",
            div {
                class: "playlistadder",
                h3 { "Add {CONTROLLER.read().all_tracks[ADD_TO_PLAYLIST().unwrap()].title} to a playlist" }

                hr {}

                for i in 0..CONTROLLER.read().playlists.len() {
                    // Add to certain playlist
                    button {
                        onclick: move |_| {
                            CONTROLLER.write().add_to_playlist(i, ADD_TO_PLAYLIST().unwrap());
                            ADD_TO_PLAYLIST.set(None);
                        },
                        "{CONTROLLER.read().playlists[i].name}"
                    }
                }

                // Only show extra separator if there are playlist buttons 
                if !CONTROLLER.read().playlists.is_empty() {
                    hr {}
                }

                // Sends user to playlist creation menu
                button { 
                    onclick: move |_| {
                        ADD_TO_PLAYLIST.set(None);
                        VIEW.write().current = View::Playlists;
                        CREATING_PLAYLIST.set(true);
                    },
                    "Create a playlist"
                }
            }
        }
    }
}
