use dioxus::prelude::*;
use dioxus::stores::SyncStore;
use crate::app::MusicController;
use crate::app::controller::MusicControllerStoreImplExt;
use crate::gui::{DELETING_TRACK, icons::*};

#[component]
pub fn DuplicateMenu(controller: SyncStore<MusicController>) -> Element {
    let mut duplicates: Signal<Vec<Vec<usize>>> = use_signal(|| Vec::new());

    rsx!{
        button {
            class: "basicbutton",
            onclick: move |_| duplicates.set(controller.find_duplicates()),
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
                        margin: "15px 0px auto 5px",
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
