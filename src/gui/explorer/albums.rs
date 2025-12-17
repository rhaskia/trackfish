use super::TracksView;
use crate::app::utils::strip_unnessecary;
use crate::{
    app::{MusicController, utils::similar},
    gui::{icons::*, View, VIEW},
};
use dioxus::document::eval;
use dioxus::prelude::*;
use super::ExplorerSwitch;

#[component]
pub fn AlbumsList(controller: SyncSignal<MusicController>) -> Element {
    let mut albums = use_signal(|| Vec::new());
    let mut is_searching = use_signal(|| false);

    use_effect(move || {
        let mut albums_unsorted = controller
            .read()
            .albums
            .clone()
            .into_iter()
            .collect::<Vec<(String, usize)>>();
        albums_unsorted.sort_by(|(_, a), (_, b)| b.cmp(a));
        albums.set(albums_unsorted);
    });

    let set_album = move |name| {
        VIEW.write().album = Some(name);
    };

    // Virtualization control
    // Works a little differently with multiple items per row
    let mut window_size = use_signal(|| 0);
    let mut items_per_row = use_signal(|| 5);
    let mut row_height = use_signal(|| 1);
    const BUFFER_ROWS: usize = 3;

    let mut start_index = use_signal(|| 0);
    let rows_in_view = use_memo(move || window_size() / row_height() + BUFFER_ROWS);
    let end_index = use_memo(move || {
        (start_index() + (rows_in_view() * items_per_row())).min(albums.read().len())
    });

    // List width and height watcher
    use_effect(move || {
        let mut js = eval(
            r#"
            new ResizeObserver(() => {
                let container = document.getElementById("albumlist");
                console.log([container.offsetHeight, container.offsetWidth]);
                dioxus.send([container.offsetHeight, container.offsetWidth]);
            }).observe(document.getElementById("albumlist"));
        "#,
        );

        spawn(async move {
            loop {
                let size = js.recv::<(usize, usize)>().await;

                if let Ok((height, width)) = size {
                    if height == 0 || width == 0 {
                        continue;
                    }
                    window_size.set(height);
                    items_per_row.set((width / 150).max(3));
                    let item_width = (width - 10) / items_per_row() - 5;
                    row_height.set(item_width + 48);
                }
            }
        });
    });

    // Watches for scroll inside list
    use_effect(move || {
        let mut js = eval(
            r#"
            let container = document.getElementById('albumlist');
            container.addEventListener('scroll', function(event) {
                dioxus.send(container.scrollTop);
            });
        "#,
        );

        spawn(async move {
            loop {
                let scroll_top = js.recv::<usize>().await;
                if let Ok(scroll_top) = scroll_top {
                    let new_index = (scroll_top as f32 / row_height() as f32).floor() as usize;
                    if new_index != start_index() {
                        start_index.set((new_index.max(1) - 1) * items_per_row());
                    }
                }
            }
        });
    });

    rsx! {
        div {
            id: "albumsview",
            position: "relative",
            class: "albums view",
            autofocus: true,
            onkeydown: move |e| log::info!("{e:?}"),
            onclick: move |_| is_searching.set(false),

            ExplorerSwitch { controller }

            div {
                class: "searchbar",
                onclick: move |e| { is_searching.set(true); e.stop_propagation()},
                display: if VIEW.read().album.is_some() { "none" },
                img { src: SEARCH_ICON }
                div { class: "pseudoinput" }
            }

            div {
                id: "albumlist",
                class: "tracklist",
                position: "relative",
                display: if VIEW.read().album.is_some() { "none" },

                div { min_height: "{row_height() * albums.read().len() / items_per_row()}px" }

                div {
                    class: "albumsholder",
                    position: "absolute",
                    top: "{row_height() * start_index() / items_per_row()}px",

                    for i in start_index()..end_index() {
                        div {
                            class: "albumitem",
                            id: "album-{albums.read()[i].0}",
                            onclick: move |_| set_album(albums.read()[i].0.clone()),

                            img {
                                loading: "onvisible",
                                src: if VIEW.read().current == View::Albums { "/trackimage/{controller.read().get_album_artwork(albums.read()[i].0.clone())}" },
                            }

                            div { class: "albuminfo",
                                if albums.read()[i].0.is_empty() {
                                    span { "Unknown Album" }
                                } else {
                                    span { "{albums.read()[i].0}" }
                                }
                                small { "{albums.read()[i].1} songs" }
                            }
                        }
                    }
                }
            }

            if VIEW.read().album.is_some() {
                TracksView { controller, viewtype: View::Albums }
            }

            if is_searching() {
                AlbumsSearch { controller, is_searching, albums, row_height, items_per_row }
            }
        }
    }
}

#[component]
pub fn AlbumsSearch(
    controller: SyncSignal<MusicController>,
    is_searching: Signal<bool>,
    albums: Signal<Vec<(String, usize)>>,
    row_height: Signal<usize>,
    items_per_row: Signal<usize>
) -> Element {
    let mut search = use_signal(String::new);

    let matches = use_memo(move || {
        let search = strip_unnessecary(&search.read());
        log::info!("searching {search}");

        if search.is_empty() {
            log::info!("searching {search}");
            Vec::new()
        } else {
            controller
                .read()
                .albums
                .iter()
                .map(|a| a.0)
                .filter(|t| strip_unnessecary(&t).starts_with(&search))
                .cloned()
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
                        value: search,
                        autofocus: true,
                        onclick: |e| e.stop_propagation(),
                        oninput: move |e| search.set(e.value()),
                    }
                }

                div { class: "searchtracks",
                    for album in matches() {
                        div {
                            class: "trackitem",
                            onclick: {
                                let album = album.clone();
                                move |_| {
                                    let index = albums.read().iter().position(|a| similar(&a.0, &album)).unwrap_or(0);
                                    let row = index / items_per_row();
                                    let scroll_amount = row * row_height();
                                    document::eval(
                                        &format!(
                                            "document.getElementById('albumlist').scrollTop = {};",
                                            scroll_amount,
                                        ),
                                    );
                                }
                            },

                            img { src: "/trackimage/{controller.read().get_album_artwork(album.clone())}?origin=albums", loading: "lazy" }
                            span { "{album}" }
                        }
                    }
                }
            }

            div { flex: 1 }
        }
    }
}
