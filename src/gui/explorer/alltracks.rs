use super::{View, VIEW};
use dioxus::prelude::*;
use dioxus::document::eval;
use log::info;
use super::TracksSearch;

fn display_time(total: u64) -> String {
    let seconds = total % 60;
    let minutes = (total % 3600 - seconds) / 60;
    let hours = (total - minutes) / 3600;

    format!("{hours}:{minutes:02}:{seconds:02}")
}

use super::CONTROLLER;

#[component]
pub fn AllTracks() -> Element {
    let mut is_searching = use_signal(|| false);
    let tracks = use_memo(move || (0..CONTROLLER.read().all_tracks.len()).collect::<Vec<usize>>());
    let total_time = use_memo(move || {
        CONTROLLER
            .read()
            .all_tracks
            .iter()
            .map(|t| t.len)
            .sum::<f64>() as u64
    });

    let mut window_size = use_signal(|| 0);
    const ROW_HEIGHT: usize = 62;
    const BUFFER_ROWS: usize = 5;

    let mut start_index = use_signal(|| 0);
    let rows_in_view = use_memo(move || window_size() / ROW_HEIGHT + BUFFER_ROWS);
    let end_index = use_memo(move || (start_index() + rows_in_view()).min(tracks.read().len()));

    use_future(move || async move {
        let mut js = eval(
            r#"
            new ResizeObserver(() => {
                let container = document.getElementById("alltrackslist");
                dioxus.send(container.offsetHeight);
            }).observe(document.getElementById("alltrackslist"));
        "#,
        );

        loop {
            let height = js.recv::<usize>().await;
            if let Ok(height) = height {
                window_size.set(height);
                info!("{height}");
            }
        }
    });

    use_effect(move || {
        let mut js = eval(
            r#"
            let container = document.getElementById('alltrackslist');
            container.addEventListener('scroll', function(event) {
                dioxus.send(container.scrollTop);
            });
        "#,
        );

        spawn(async move {
            loop {
                let scroll_top = js.recv::<usize>().await;
                if let Ok(scroll_top) = scroll_top {
                    let new_index = (scroll_top as f32 / ROW_HEIGHT as f32).floor() as usize;
                    if new_index != start_index() {
                        start_index.set(new_index);
                    }
                }
            }
        });
    });

    rsx! {
        div {
            class: "alltracksview",
            display: if VIEW.read().current != View::AllTracks { "none" },
            div { class: "searchbar", onclick: move |_| is_searching.set(true),
                img { src: "assets/icons/search.svg" }
                div { class: "pseudoinput" }
            }
            div { color: "white", padding: "10px",
                "{CONTROLLER.read().all_tracks.len()} songs / "
                "{display_time(total_time())} total duration"
            }
            div {
                class: "tracklist",
                id: "alltrackslist",
                position: "relative",

                div { min_height: "{(tracks.read().len()) * ROW_HEIGHT}px" }

                for i in start_index()..end_index() {
                    div {
                        class: "trackitem",
                        id: "alltracks-trackitem-{i}",
                        style: "top: {i * ROW_HEIGHT}px; position: absolute;",
                        onclick: move |_| {
                            CONTROLLER.write().add_all_queue(i);
                            VIEW.write().current = View::Song;
                        },
                        img {
                            class: "trackitemicon",
                            loading: "onvisible",
                            src: "/trackimage/{i}",
                        }
                        span { "{CONTROLLER.read().all_tracks[i].title}" }
                        div { flex_grow: 1 }
                        img {
                            class: "trackbutton",
                            loading: "onvisible",
                            src: "/assets/icons/vert.svg",
                        }
                    }
                }
            
            }
            if is_searching() {
                TracksSearch { tracks, is_searching, id_prefix: "alltracks" }
            }
        }
    }
}

