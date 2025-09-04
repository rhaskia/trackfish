use super::explorer::TracksView;
use super::{Confirmation, View, ADD_TO_PLAYLIST, VIEW};
use crate::app::MusicController;
use crate::app::playlist::Playlist;
use dioxus::prelude::*;

const CREATING_PLAYLIST: GlobalSignal<bool> = Signal::global(|| false);

use super::icons::*;

#[component]
pub fn PlaylistsView(controller: SyncSignal<MusicController>) -> Element {
    let mut playlist_name = use_signal(String::new);
    let mut playlist_options = use_signal(|| None);
    let mut deleting_playlist = use_signal(|| None);
    let renaming_playlist = use_signal(|| None);

    rsx! {
        div {
            class: "playlistsview",
            display: if VIEW.read().current != View::Playlists { "none" },
            div { padding: "10px", hidden: VIEW.read().playlist.is_some(),
                h3 { "Playlists" }
                hr {}

                // Playlist list
                for i in 0..controller.read().playlists.len() {
                    div {
                        class: "playlistitem",
                        onclick: move |_| VIEW.write().playlist = Some(i),
                        img { src: PLAYLIST_PLAY_ICON }
                        "{controller.read().playlists[i].name}"
                        div { flex: "1 1 0" }
                        img {
                            onclick: move |e| {
                                e.stop_propagation();
                                playlist_options.set(Some(i));
                            },
                            src: VERT_ICON,
                        }
                    }
                }

                button { onclick: move |_| *CREATING_PLAYLIST.write() = true, "Create new playlist" }

                // Player creation menu
                div {
                    class: "playlistcreatorbg",
                    hidden: !CREATING_PLAYLIST(),
                    onclick: move |_| *CREATING_PLAYLIST.write() = false,
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
                                let dir = controller.write().settings.directory.clone();
                                controller.write().playlists.push(Playlist::new(playlist_name(), dir));
                                *CREATING_PLAYLIST.write() = false;
                                playlist_name.set(String::new());
                            },
                            disabled: playlist_name.read().is_empty(),
                            "Create"
                        }
                    }
                }
            }

            if VIEW.read().playlist.is_some() {
                TracksView { controller, viewtype: View::Playlists }
            }
        }

        if ADD_TO_PLAYLIST.read().is_some() {
            PlaylistAdder { controller }
        }

        if playlist_options.read().is_some() && VIEW.read().current == View::Playlists {
            PlaylistOptions {
                controller,
                playlist_options,
                deleting_playlist,
                renaming_playlist,
            }
        }

        if renaming_playlist.read().is_some() {
            PlaylistRename { controller, renaming_playlist }
        }

        if deleting_playlist.read().is_some() {
            Confirmation {
                label: "Delete playlist {controller.read().playlists[deleting_playlist().unwrap()].name}?",
                confirm: move |_| controller.write().delete_playlist(deleting_playlist().unwrap()),
                cancel: move |_| deleting_playlist.set(None),
            }
        }
    }
}

#[component]
pub fn PlaylistRename(controller: SyncSignal<MusicController>,  renaming_playlist: Signal<Option<usize>>) -> Element {
    let mut new_name = use_signal(String::new);
    rsx! {
        div { class: "optionsbg", onclick: move |_| renaming_playlist.set(None),
            div { class: "playlistadder",
                input {
                    r#type: "text",
                    onclick: |e| e.stop_propagation(),
                    onchange: move |e| new_name.set(e.data.value()),
                }
                button {
                    onclick: move |_| {
                        controller.write().playlists[renaming_playlist().unwrap()].name = new_name();
                        controller.write().save_playlist(renaming_playlist().unwrap());
                    },
                    "Rename"
                }
            }
        }
    }
}

#[component]
pub fn PlaylistAdder(controller: SyncSignal<MusicController>) -> Element {
    rsx! {
        div {
            class: "playlistadderbg",
            onclick: move |_| *ADD_TO_PLAYLIST.write() = None,
            div { class: "playlistadder",
                h3 {
                    "Add {controller.read().all_tracks[ADD_TO_PLAYLIST().unwrap()].title} to a playlist"
                }

                for i in 0..controller.read().playlists.len() {
                    // Add to certain playlist
                    button {
                        onclick: move |_| {
                            controller.write().add_to_playlist(i, ADD_TO_PLAYLIST().unwrap());
                            *ADD_TO_PLAYLIST.write() = None;
                        },
                        "{controller.read().playlists[i].name}"
                    }
                }

                // Only show extra separator if there are playlist buttons
                if !controller.read().playlists.is_empty() {
                    hr {}
                }

                // Sends user to playlist creation menu
                button {
                    onclick: move |_| {
                        *ADD_TO_PLAYLIST.write() = None;
                        VIEW.write().current = View::Playlists;
                        *CREATING_PLAYLIST.write() = true;
                    },
                    "Create a playlist"
                }
            }
        }
    }
}

#[component]
pub fn PlaylistOptions(
    controller: SyncSignal<MusicController>,
    playlist_options: Signal<Option<usize>>,
    deleting_playlist: Signal<Option<usize>>,
    renaming_playlist: Signal<Option<usize>>,
) -> Element {
    rsx! {
        div { class: "optionsbg", onclick: move |_| playlist_options.set(None),
            div { class: "optionbox", style: "--width: 300px; --height: 50px;",
                h3 { "{controller.read().playlists[playlist_options().unwrap()].name}" }
                button { onclick: move |_| renaming_playlist.set(playlist_options()),
                    img { src: EDIT_ICON }
                    "Rename playlist"
                }
                button {
                    img { src: EXPORT_ICON }
                    "Export playlist"
                }
                button { onclick: move |_| deleting_playlist.set(playlist_options()),
                    img { src: DELETE_ICON }
                    "Delete playlist"
                }
            }
        }
    }
}
