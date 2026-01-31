use super::TracksSearch;
use super::{View, VIEW};
use crate::app::MusicController;
use crate::app::controller::MusicControllerStoreExt;
use crate::gui::icons::*;
use dioxus::document::eval;
use dioxus::prelude::*;
use dioxus::stores::SyncStore;
use log::info;

fn display_time(total: u64) -> String {
    let seconds = total % 60;
    let minutes = (total % 3600 - seconds) / 60;
    let hours = (total - minutes) / 3600;

    format!("{hours}:{minutes:02}:{seconds:02}")
}

#[component]
pub fn AllTracks(controller: SyncStore<MusicController>) -> Element {
    let mut is_searching = use_signal(|| false);
    
    let tracks = use_memo(move || (0..controller.all_tracks().read().len()).collect::<Vec<usize>>());
    let total_time = use_memo(move || {
        controller
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
                info!("window height {height}");
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
                        info!("{start_index:?}..{end_index:?}");
                    }
                }
            }
        });
    });

    rsx! {
        div { id: "alltracksview", class: "alltracksview view",
            div { class: "searchbar", onclick: move |_| is_searching.set(true),
                img { src: SEARCH_ICON, loading: "lazy" }
                div { class: "pseudoinput" }
            }
            div { color: "white", padding: "10px",
                "{controller.all_tracks().read().len()} songs / "
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
                            controller.write().add_all_queue(i);
                            VIEW.write().current = View::Song;
                        },
                        img {
                            class: "trackitemicon",
                            loading: "onvisible",
                            src: if VIEW.read().current == View::AllTracks { "/trackimage/{i}?origin=alltracks" },
                        }
                        span { "{controller.all_tracks().get(i).unwrap().read().title}" }
                        div { flex_grow: 1 }
                        img {
                            class: "trackbutton",
                            loading: "lazy",
                            src: VERT_ICON,
                        }
                    }
                }
            }

            if is_searching() {
                TracksSearch {
                    controller,
                    tracks,
                    is_searching,
                    id_prefix: "alltrackslist",
                }
            }
        }
    }
}
