use super::TracksView;
use crate::{
    app::MusicController,
    gui::{icons::*, View, VIEW, SEARCHER},
};
use dioxus::prelude::*;
use super::ExplorerSwitch;

#[component]
pub fn ArtistList(controller: SyncSignal<MusicController>) -> Element {
    let mut artists = use_signal(|| Vec::new());
    let mut is_searching = use_signal(|| false);
    let mut set_searcher_artists = use_signal(|| false);

    use_effect(move || {
        let mut artists_unsorted = controller
            .read()
            .artists
            .clone()
            .into_iter()
            .collect::<Vec<(String, (String, usize))>>();
        artists_unsorted.sort_by(|(_, (_, a)), (_, (_, b))| b.cmp(a));
        artists.set(artists_unsorted);
    });

    use_effect(move || {
        if artists.read().len() > 0 && !set_searcher_artists() {
            SEARCHER.write().fill_artist_information(&*artists.read());
            set_searcher_artists.set(true);
        }
    });

    let set_artist = move |name| {
        VIEW.write().artist = Some(name);
    };

    rsx! {
        div { id: "artistsview", class: "artists view",

            ExplorerSwitch { controller }

            div {
                class: "searchbar",
                display: if VIEW.read().artist.is_some() { "none" },
                onclick: move |_| is_searching.set(true),

                img { src: SEARCH_ICON }

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
                TracksView { controller, viewtype: View::Artists }
            }

            if is_searching() {
                ArtistsSearch { controller, is_searching, artists }
            }
        }
    }
}

#[component]
pub fn ArtistsSearch(
    controller: SyncSignal<MusicController>,
    is_searching: Signal<bool>,
    artists: Signal<Vec<(String, (String, usize))>>,
) -> Element {
    let mut search = use_signal(String::new);

    let matches = use_memo(move || {
        log::info!("searching {search}");

        if search.len() <= 1 {
            Vec::new()
        } else {
            SEARCHER.write().search_artists(&*search.read())
        }
    });

    let row_height = 58;

    rsx! {
        div { class: "searchholder", onclick: move |_| is_searching.set(false),
            div { flex: 1 }

            div { class: "searchpopup",
                div { class: "searchpopupbar",
                    img { src: SEARCH_ICON }

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
                                // Requires the scroll amount to be one less height than that of the object to actually show it
                                let index = artists.read().iter().position(|a| a.1.0 == artist).unwrap();
                                let scroll_amount = index * row_height;

                                info!("scrolling {scroll_amount} for index {index}");
                                document::eval(
                                    &format!(
                                        "document.getElementById('artistlist').scrollTop = {};",
                                        scroll_amount,
                                    ),
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
