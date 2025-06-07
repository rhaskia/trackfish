use dioxus::prelude::*;
use crate::{View, TRACKOPTION, VIEW, CONTROLLER};
use super::TracksView;
use app::utils::strip_unnessecary;

#[component]
pub fn SearchView() -> Element {
    rsx!{
        div {
            class: "searchview",
            display: if VIEW.read().current != View::Search { "none" },
            div {
                class: "searchbar",
                display: if VIEW.read().genre.is_some() { "none" },
                img { src: "assets/icons/search.svg" }
                input {}
            }
        }
    }
}

#[component]
pub fn TracksSearch(tracks: Memo<Vec<usize>>, is_searching: Signal<bool>, id_prefix: String) -> Element {
    let mut search = use_signal(String::new);
    let id_prefix = use_signal(|| id_prefix);
    
    let matches = use_memo(move || {
        let search = strip_unnessecary(&search.read());
        log::info!("searching {search}");

        if search.is_empty() {
            Vec::new()
        } else {
            tracks
                .read()
                .iter()
                .filter(|t| {
                    strip_unnessecary(&CONTROLLER.read().all_tracks[**t].title).starts_with(&search)
                })
                .cloned()
                .collect::<Vec<usize>>()
        }
    });

    rsx! {
        div { class: "searchholder", onclick: move |_| is_searching.set(false),
            div { flex: 1 }
            div { class: "searchpopup",
                div { class: "searchpopupbar",
                    img { src: "assets/icons/search.svg" }
                    input {
                        value: search,
                        autofocus: true,
                        onclick: |e| e.stop_propagation(),
                        oninput: move |e| search.set(e.value()),
                    }
                }
                div { class: "searchtracks",
                    for track in matches() {
                        div {
                            class: "trackitem",
                            onclick: move |_| {
                                document::eval(
                                    &format!("document.getElementById('{id_prefix}-trackitem-{}').scrollIntoView();", track),
                                );
                            },
                            img { src: "/trackimage/{track}" }
                            span { "{CONTROLLER.read().all_tracks[track].title}" }
                        }
                    }
                }
            }
            div { flex: 1 }
        }
    }
}
