use super::TracksView;
use crate::app::controller::{MusicControllerStoreExt, MusicControllerStoreImplExt};
use crate::app::utils::title_case;
use crate::{
    app::MusicController,
    gui::{icons::*, View, VIEW, SEARCHER, Confirmation},
};
use dioxus::prelude::*;
use dioxus::stores::SyncStore;
use super::ExplorerSwitch;

pub const RENAMING_GENRE: GlobalSignal<bool> = Signal::global(|| false);
pub const DELETING_GENRE: GlobalSignal<bool> = Signal::global(|| false);

#[component]
pub fn GenreList(controller: SyncStore<MusicController>) -> Element {
    let mut genres: Signal<Vec<(String, (String, usize))>> = use_signal(|| Vec::new());
    let mut is_searching = use_signal(|| false);
    let mut set_searcher_genres = use_signal(|| false);
    let mut new_genre_name = use_signal(String::new);

    use_effect(move || {
        if genres.read().len() > 0 && !set_searcher_genres() {
            SEARCHER.write().fill_genre_information(&*genres.read());
            set_searcher_genres.set(true);
        }
    });

    use_effect(move || {
        let mut genres_unsorted = controller
            .genres()()
            .into_iter()
            .collect::<Vec<(String, (String, usize))>>();
        genres_unsorted.sort_by(|(_, (_, a)), (_, (_, b))| b.cmp(a));
        genres.set(genres_unsorted);
    });

    let set_genre = move |name| {
        VIEW.write().genre = Some(name);
    };

    let mut row_height = use_signal(|| 39i32);

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
        div { class: "artists view", id: "genreview",

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
                    if genres.read()[i].1.1 > 1 {
                        div {
                            class: "thinitem",
                            id: "genre-{genres.read()[i].0}",
                            onclick: move |_| set_genre(title_case(&genres.read()[i].1.0)),
                            if genres.read()[i].0.is_empty() {
                                "Unknown Genres"
                            } else {
                                "{title_case(&genres.read()[i].1.0)}"
                            }
                            small { "{genres.read()[i].1.1} songs" }
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

            if RENAMING_GENRE() {
                div { class: "optionsbg", onclick: move |_| RENAMING_GENRE.set(false),
                    div { class: "playlistadder",
                        input {
                            r#type: "text",
                            onclick: |e| e.stop_propagation(),
                            onchange: move |e| new_genre_name.set(e.data.value()),
                        }

                        button {
                            onclick: move |_| {
                                let old_name = VIEW.read().genre.clone().unwrap();
                                controller.rename_genre(old_name, new_genre_name());
                                VIEW.write().genre = Some(title_case(&new_genre_name()));
                            },
                            "Rename"
                        }
                    }
                }
            }

            Confirmation {
                label: "Delete genre {VIEW.read().genre.clone().unwrap_or_default()}?",
                confirm: move |_| {
                    controller.delete_genre(VIEW.read().genre.clone().unwrap());
                    DELETING_GENRE.set(false);
                    VIEW.write().genre = None;
                },
                cancel: move |_| DELETING_GENRE.set(false),
                visible: DELETING_GENRE(),
            }
        }
    }
}

#[component]
pub fn GenreSearch(is_searching: Signal<bool>, genres: Signal<Vec<(String, (String, usize))>>, row_height: Signal<i32>) -> Element {
    let mut search = use_signal(String::new);

    let matches = use_memo(move || {
        log::info!("searching {search}");

        if search.len() <= 1 {
            Vec::new()
        } else {
            SEARCHER.write().search_genres(&*search.read())
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
                                let index = genres.read().iter().position(|a| a.0 == genre).unwrap();
                                let scroll_amount = index as i32 * row_height();

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

#[component]
pub fn GenreOptions(controller: SyncStore<MusicController>) -> Element {
    rsx!{
        hr { }

        button { onclick: move |_| RENAMING_GENRE.set(true), // Possibly use a name string later on
            img { src: EDIT_ICON }
            "Rename Genre"
        }

        button { onclick: move |_| DELETING_GENRE.set(true), 
            img { src: DELETE_ICON }
            "Delete Genre"
        }
    }
}