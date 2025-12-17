use super::TracksView;
use crate::app::utils::strip_unnessecary;
use crate::{
    app::MusicController,
    gui::{icons::*, View, VIEW},
};
use dioxus::prelude::*;
use super::ExplorerSwitch;

#[component]
pub fn GenreList(controller: SyncSignal<MusicController>) -> Element {
    let mut genres = use_signal(|| Vec::new());
    let mut is_searching = use_signal(|| false);

    use_effect(move || {
        let mut genres_unsorted = controller
            .read()
            .genres
            .clone()
            .into_iter()
            .collect::<Vec<(String, usize)>>();
        genres_unsorted.sort_by(|(_, a), (_, b)| b.cmp(a));
        genres.set(genres_unsorted);
    });

    let set_genre = move |name| {
        VIEW.write().genre = Some(name);
    };

    let mut row_height = use_signal(|| 10i32);

    use_future(move || async move {
        tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.25)).await;

        let mut js = dioxus::document::eval(r#"
            dioxus.send(document.getElementById('genrelist).firstChild.clientHeight);
        "#);

        let rh_maybe = js.recv().await;
        info!("genre rh {rh_maybe:?}");
        if let Ok(rh) = rh_maybe {
            row_height.set(rh);
            info!("genre row_height found as {rh}");
        }
    });

    rsx! {
        div {
            class: "artists view",
            id: "genreview",

            ExplorerSwitch { controller }

            div {
                class: "searchbar",
                display: if VIEW.read().genre.is_some() { "none" },
                onclick: move |_| is_searching.set(true),
                img { src: SEARCH_ICON }
                div { class: "pseudoinput" }
            }

            div {
                id: "genrelist",
                class: "tracklist",
                display: if VIEW.read().genre.is_some() { "none" },

                for i in 0..genres.read().len() {
                    if genres.read()[i].1 > 1 {
                        div {
                            class: "thinitem",
                            id: "genre-{genres.read()[i].0}",
                            onclick: move |_| set_genre(genres.read()[i].0.clone()),
                            if genres.read()[i].0.is_empty() {
                                "Unknown Genres"
                            } else {
                                "{genres.read()[i].0}"
                            }
                            small { "{genres.read()[i].1} songs" }
                        }
                    }
                }
            }

            if VIEW.read().genre.is_some() {
                TracksView { controller, viewtype: View::Genres }
            }

            if is_searching() {
                GenreSearch { is_searching, genres, row_height }
            }
        }
    }
}

#[component]
pub fn GenreSearch(is_searching: Signal<bool>, genres: Signal<Vec<(String, usize)>>, row_height: Signal<i32>) -> Element {
    let mut search = use_signal(String::new);

    let matches = use_memo(move || {
        let search = strip_unnessecary(&search.read());
        log::info!("searching {search}");

        if search.is_empty() {
            Vec::new()
        } else {
            genres
                .read()
                .iter()
                .map(|t| t.0.clone())
                .filter(|t| strip_unnessecary(&t).starts_with(&search))
                .collect::<Vec<String>>()
        }
    });

    rsx! {
        div { class: "searchholder", onclick: move |_| is_searching.set(false),
            div { flex: 1 }

            div { class: "searchpopup",
                div { class: "searchpopupbar",
                    img { src: SEARCH_ICON }

                    input {
                        id: "genresearchbar",
                        value: search,
                        autofocus: true,
                        onclick: |e| e.stop_propagation(),
                        oninput: move |e| search.set(e.value()),
                    }
                }

                div { class: "searchtracks",
                    for genre in matches() {
                        div {
                            class: "thinitem",
                            onclick: move |_| {
                                // Requires the scroll amount to be one less height than that of the object to actually show it
                                let scroll_amount = (genres.read().iter().position(|a| a.0 == genre).unwrap().max(1) - 1) as i32 * row_height();

                                document::eval(
                                    &format!(
                                        "document.getElementById('genrelist').scrollTop = {};",
                                        scroll_amount,
                                    ),
                                );
                            },

                            span { "{genre}" }
                        }
                    }
                }
            }

            div { flex: 1 }
        }
    }
}
