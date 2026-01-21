use dioxus::prelude::*;
use crate::app::MusicController;
use super::DELETING_TRACK;
use super::icons::*;
use super::explorer::ExplorerSwitch;

pub enum LibraryMenu {
    Duplicates,
    Autotagging
}

#[component]
pub fn LibraryManagement(controller: SyncSignal<MusicController>) -> Element {
    let mut menu = use_signal(|| LibraryMenu::Duplicates);
    
    rsx!{
        ExplorerSwitch { controller }

        div { class: "librarymanagementview view", id: "librarymanagement",
            div { class: "header",
                button {
                    class: "basicbutton",
                    onclick: move |_| menu.set(LibraryMenu::Duplicates),
                    "Duplicates"
                }
                button {
                    class: "basicbutton",
                    onclick: move |_| menu.set(LibraryMenu::Autotagging),
                    "Autotagging"
                }
            }

            match *menu.read() {
                LibraryMenu::Duplicates => rsx! {
                    DuplicateMenu { controller }
                },
                LibraryMenu::Autotagging => rsx! {
                    TaggingMenu { controller }
                },
            }
        }
    }
}

#[component]
pub fn DuplicateMenu(controller: SyncSignal<MusicController>) -> Element {
    let mut duplicates: Signal<Vec<Vec<usize>>> = use_signal(|| Vec::new());

    rsx!{
        button {
            class: "basicbutton",
            onclick: move |_| duplicates.set(controller.read().find_duplicates()),
            margin: "0 10px",
            "Load duplicates"
        }
        div { class: "duplicates",
            for i in 0..duplicates.read().len() {
                div { class: "duplicategroup",
                    img {
                        onclick: move |_| {
                            duplicates.remove(i);
                        },
                        margin: "15px 0px auto 5px;",
                        margin_bottom: "auto",
                        class: "trackbutton",
                        loading: "lazy",
                        src: CLOSE_ICON,
                    }
                    div { class: "duplicatesongs",
                        for j in 0..duplicates.read()[i].len().min(20) {
                            div { class: "duplicatesong",
                                img {
                                    class: "trackitemicon",
                                    loading: "onvisible",
                                    src: "/trackimage/{duplicates.read()[i][j]}?origin=library",
                                }
                                "{controller.read().all_tracks[duplicates.read()[i][j]].title} - "
                                "{controller.read().all_tracks[duplicates.read()[i][j]].artists.join(\", \")}"
                                div { flex_grow: 1 }
                                img {
                                    onclick: move |_| DELETING_TRACK.set(Some(duplicates.read()[i][j])),
                                    class: "trackbutton",
                                    loading: "lazy",
                                    src: DELETE_ICON,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn TaggingMenu(controller: SyncSignal<MusicController>) -> Element {
    rsx!{}
}