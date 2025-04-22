use super::explorer::TracksView;
use super::{Confirmation, View, ADD_TO_PLAYLIST, CONTROLLER, VIEW};
use crate::app::playlist::Playlist;
use dioxus::prelude::*;

const CREATING_PLAYLIST: GlobalSignal<bool> = Signal::global(|| false);

#[component]
pub fn PlaylistsView() -> Element {
    let mut playlist_name = use_signal(String::new);
    let mut playlist_options = use_signal(|| None);
    let mut deleting_playlist = use_signal(|| None);
    let mut renaming_playlist = use_signal(|| None);

    rsx! {
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
                        img {
                            onclick: move |e| {
                                e.stop_propagation();
                                playlist_options.set(Some(i));
                            },
                            src: "assets/icons/vert.svg"
                        }
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

        if playlist_options.read().is_some() && VIEW.read().current == View::Playlists {
            PlaylistOptions { playlist_options, deleting_playlist, renaming_playlist }
        }

        if renaming_playlist.read().is_some() {
            PlaylistRename { renaming_playlist }
        }

        if deleting_playlist.read().is_some() {
            Confirmation {
                label: "Delete playlist {CONTROLLER.read().playlists[deleting_playlist().unwrap()].name}?",
                confirm: move |_| CONTROLLER.write().delete_playlist(deleting_playlist().unwrap()),
                cancel: move |_| deleting_playlist.set(None),
            }
        }
    }
}

#[component]
pub fn PlaylistRename(mut renaming_playlist: Signal<Option<usize>>) -> Element {
    let mut new_name = use_signal(String::new);
    rsx!{
        div {
            class: "optionsbg",
            onclick: move |_| renaming_playlist.set(None),
            div {
                class: "playlistadder",
                input {
                    r#type: "text",
                    onclick: |e| e.stop_propagation(),
                    onchange: move |e| new_name.set(e.data.value()),
                }
                button {
                    onclick: move |_| {
                        CONTROLLER.write().playlists[renaming_playlist().unwrap()].name = new_name();
                        CONTROLLER.write().save_playlist(renaming_playlist().unwrap());
                    },
                    "Rename"
                }
            }
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

#[component]
pub fn PlaylistOptions(
    playlist_options: Signal<Option<usize>>,
    deleting_playlist: Signal<Option<usize>>,
    renaming_playlist: Signal<Option<usize>>,
) -> Element {
    rsx! {
        div {
            class: "optionsbg",
            onclick: move |_| playlist_options.set(None),
            div {
                class: "optionbox",
                style: "--width: 300px; --height: 50px;",
                h3 { "{CONTROLLER.read().playlists[playlist_options().unwrap()].name}" }
                button {
                    onclick: move |_| renaming_playlist.set(playlist_options()),
                    img { src: "assets/icons/edit.svg" }
                    "Rename playlist"
                }
                button {
                    img { src: "assets/icons/export.svg" }
                    "Export playlist"
                }
                button {
                    onclick: move |_| deleting_playlist.set(playlist_options()),
                    img { src: "assets/icons/delete.svg" }
                    "Delete playlist"
                }
            }
        }
    }
}
