use dioxus::prelude::*;
use crate::gui::{View, VIEW, CONTROLLER};
use super::TracksView;
use crate::app::utils::strip_unnessecary;

#[component]
pub fn ArtistList() -> Element {
    let mut artists = use_signal(|| Vec::new());
    let mut is_searching = use_signal(|| false);

    use_effect(move || {
        let mut artists_unsorted = CONTROLLER
            .read()
            .artists
            .clone()
            .into_iter()
            .collect::<Vec<(String, (String, usize))>>();
        artists_unsorted.sort_by(|(_, (_, a)), (_, (_, b))| b.cmp(a));
        artists.set(artists_unsorted);
    });

    let set_artist = move |name| {
        VIEW.write().artist = Some(name);
    };

    rsx! {
        div {
            class: "artists",
            display: if VIEW.read().current != View::Artists { "none" },
            div {
                class: "searchbar",
                display: if VIEW.read().artist.is_some() { "none" },
                onclick: move |_| is_searching.set(true),
                img { src: "assets/icons/search.svg" }
                div { class: "pseudoinput" }
            }
            div {
                id: "artistlist",
                class: "tracklist",
                display: if VIEW.read().artist.is_some() { "none" },

                for i in 0..artists.read().len() {
                    div {
                        class: "thinitem",
                        id: "artist-{artists.read()[i].1.0}",
                        onclick: move |_| set_artist(artists.read()[i].clone().1.0),
                        "{artists.read()[i].1.0}"
                        br {}
                        small { "{artists.read()[i].1.1} songs" }
                    }
                }
            }
            if VIEW.read().artist.is_some() {
                TracksView { viewtype: View::Artists }
            }
            if is_searching() {
                ArtistsSearch { is_searching, artists }
            }
        }
    }
}

#[component]
pub fn ArtistsSearch(is_searching: Signal<bool>, artists: Signal<Vec<(String, (String, usize))>>) -> Element {
    let mut search = use_signal(String::new);
    
    let matches = use_memo(move || {
        let search = strip_unnessecary(&search.read());
        log::info!("searching {search}");

        if search.is_empty() {
            Vec::new()
        } else {
            artists
                .read()
                .iter()
                .map(|t| t.1.0.clone())
                .filter(|t| {
                    strip_unnessecary(&t).starts_with(&search)
                })
                .collect::<Vec<String>>()
        }
    });

    rsx!{
        div { class: "searchholder", onclick: move |_| is_searching.set(false),
            div { flex: 1 }
            div { class: "searchpopup",
                div { class: "searchpopupbar",
                    img { src: "assets/icons/search.svg" }
                    input {
                        id: "artistsearchbar",
                        value: search,
                        autofocus: true,
                        onclick: |e| e.stop_propagation(),
                        oninput: move |e| search.set(e.value()),
                    }
                }
                div { class: "searchtracks",
                    for artist in matches() {
                        div {
                            class: "thinitem",
                            onclick: move |_| {
                                document::eval(
                                    &format!("document.getElementById('artist-{}').scrollIntoView();", artist),
                                );
                            },
                            span { "{artist}" }
                        }
                    }
                }
            }
            div { flex: 1 }
        }
    }
}
