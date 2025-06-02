use dioxus::prelude::*;
use crate::gui::{View, TRACKOPTION, VIEW, CONTROLLER};
use super::TracksView;
use dioxus::document::eval;
use super::TracksSearch;

#[component]
pub fn AlbumsList() -> Element {
    let mut albums = use_signal(|| Vec::new());
    let mut is_searching = use_signal(|| false);

    use_effect(move || {
        let mut albums_unsorted = CONTROLLER
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

    let mut window_size = use_signal(|| 0);
    let mut items_per_row = use_signal(|| 5);
    let mut row_height = use_signal(|| 1);
    const BUFFER_ROWS: usize = 3;

    let mut start_index = use_signal(|| 0);
    let rows_in_view = use_memo(move || window_size() / row_height() + BUFFER_ROWS);
    let end_index = use_memo(move || (start_index() + (rows_in_view() * items_per_row())).min(albums.read().len()));
    let mut scroll = use_signal(|| 0);

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
                    if height == 0 || width == 0 { continue; }
                    window_size.set(height);
                    items_per_row.set((width / 150).max(3));
                    let item_width = (width - 10) / items_per_row() - 5;
                    row_height.set(item_width + 48);
                }
            }
        });
    });

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
            class: "albums",
            display: if VIEW.read().current != View::Albums { "none" },
            autofocus: true,
            onkeydown: move |e| log::info!("{e:?}"),
            onclick: move |_| is_searching.set(false),
            div { class: "searchbar", onclick: move |_| is_searching.set(true),
                img { src: "assets/icons/search.svg" }
                div { class: "pseudoinput" }
            }
            div {
                id: "albumlist",
                class: "tracklist",
                position: "relative",
                display: if VIEW.read().album.is_some() { "none" },

                div { min_height: "{row_height() * albums.read().len()}px" }

                div {
                    class: "albumsholder",
                    position: "absolute",
                    top: "{row_height() * start_index() / items_per_row()}px",

                    for i in start_index()..end_index() {
                        div {
                            class: "albumitem",
                            onclick: move |_| set_album(albums.read()[i].0.clone()),
                            img { 
                                loading: "onvisible",
                                src: "/trackimage/{CONTROLLER.read().get_album_artwork(albums.read()[i].0.clone())}"
                            }
                            div {
                                class: "albuminfo",
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
                TracksView { viewtype: View::Albums }
            }
        }
    }
}


