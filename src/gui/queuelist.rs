use super::icons::*;
use super::{View, TRACKOPTION, VIEW};
use crate::app::MusicController;
use dioxus::document::eval;
use dioxus::prelude::*;
use dioxus::stores::SyncStore;
use std::time::Duration;
use crate::app::controller::{MusicControllerStoreExt, MusicControllerStoreImplExt};

#[component]
pub fn QueueList(controller: SyncStore<MusicController>) -> Element {
    let mut selected_queue = use_signal(|| 0);
    let mut current_dragging = use_signal(|| None);
    let mut mouse_y = use_signal(|| 0.0);
    let mut scroll_y = use_signal(|| 0.0);
    let grab_y = use_signal(|| 0.0);
    let mut hovering_over = use_signal(|| 0);
    let mut queue_height = use_signal(|| 0.0);
    let mut queue_editing = use_signal(|| None);

    let mut window_size = use_signal(|| 0);
    const ROW_HEIGHT: usize = 62;
    const BUFFER_ROWS: usize = 5;

    let mut start_index = use_signal(|| 0);
    let queue_len = use_memo(move ||
        controller.queues().get(selected_queue()).unwrap().read().len()
    );
    let rows_in_view = use_memo(move || window_size() / ROW_HEIGHT + BUFFER_ROWS);
    let end_index = use_memo(move || (start_index() + rows_in_view()).min(queue_len()));

    use_effect(move || {
        selected_queue.set(controller.current_queue_index()());
    });

    use_future(move || async move {
        // Listens to js for mouse movement over whole document
        let mut js = eval(
            r#"
            document.addEventListener('mousemove', function(event) {
                dioxus.send(event.clientY);
            });
        "#,
        );

        // Calculates if the mouse if hovering over a specific track in queue
        // Could use something better than crude calculations
        loop {
            let position = js.recv::<i32>().await;
            if let Ok(pos) = position {
                mouse_y.set(pos as f32);
                hovering_over
                    .set(((mouse_y() + scroll_y()) / (ROW_HEIGHT as f32) - 0.5).floor() as usize);
            }
        }
    });

    // Watches for resize or mouse move over the queuelist
    // This makes sure that we have the current queue height whenever a calculation is needed to be made
    use_future(move || async move {
        let mut js = eval(
            r#"
            document.addEventListener('mousemove', function(event) {
                let container = document.getElementById('queuelist');
                dioxus.send(container.offsetHeight);
            });
            addEventListener('resize', function(event) {
                let container = document.getElementById('queuelist');
                dioxus.send(container.offsetHeight);
            });
        "#,
        );

        loop {
            let height = js.recv::<i32>().await;
            if let Ok(height) = height {
                queue_height.set(height as f32);
            }
        }
    });

    // Scroll up or down if mouse is hovering close to edge of queue top or bottom
    use_future(move || async move {
        loop {
            if mouse_y() < 100.0 && current_dragging.read().is_some() {
                eval("document.getElementById('queuelist').scrollBy(0, -10)");
            }
            if mouse_y() > queue_height() && current_dragging.read().is_some() {
                eval("document.getElementById('queuelist').scrollBy(0, 10)");
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    });

    // Sends the amount scrolled from top of queue on user or device scroll
    use_future(move || async move {
        let mut js = eval(
            r#"
            let container = document.getElementById('queuelist');
            container.addEventListener('scroll', function(event) {
                console.log("scroll");
                dioxus.send(event.target.scrollTop);
            });
            addEventListener("scroll", (event) => {
                dioxus.send(container.scrollTop);
            });
        "#,
        );

        // Calculates if hovering over a item in queue
        // Needs to be moved into function or joined with other future
        loop {
            let scroll = js.recv::<i32>().await;
            if let Ok(scroll) = scroll {
                scroll_y.set(scroll as f32);
                hovering_over
                    .set((((mouse_y() + scroll_y()) - 31.0) / 62.0).floor() as usize);

                let new_index = (scroll as f32 / ROW_HEIGHT as f32).floor() as usize;
                if new_index != start_index() {
                    start_index.set(new_index);
                    info!("{start_index:?}..{end_index:?}");
                }
            }
        }
    });

    // Callback for moving item into other place in queue
    let move_queue_item = move |_: Event<MouseData>| {
        if let Some(current) = current_dragging() {
            controller.queues().get(selected_queue()).unwrap().write().swap(current, hovering_over())
        }
        current_dragging.set(None);
    };

    use_future(move || async move {
        let mut js = eval(
            r#"
            new ResizeObserver(() => {
                let container = document.getElementById("queuelist");
                dioxus.send(container.offsetHeight);
            }).observe(document.getElementById("queuelist"));
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

    rsx! {
        div {
            id: "queueview",
            class: "queue view",
            onclick: move_queue_item.clone(),

            // Queue selector
            div { class: "queuebar",
                div { class: "selectwrapper queueselectwrapper",
                    select {
                        class: "queueselect",
                        onchange: move |e| selected_queue.set(e.value().parse::<usize>().unwrap()),
                        for i in 0..controller.queues().read().len() {
                            option { value: "{i}", selected: i == selected_queue(),
                                "{controller.queues().get(i).unwrap().read().queue_type}"
                            }
                        }
                    }
                }
                img {
                    onclick: move |e| {
                        e.stop_propagation();
                        queue_editing.set(Some(selected_queue()));
                    },
                    src: VERT_ICON,
                }
            }

            // Current track out of track count in queue
            span { margin: "2px 10px",
                "Track: {controller.queues().get(selected_queue()).unwrap().read().current_track + 1}/{queue_len()}"
            }

            // Track items in selected queue
            div { id: "queuelist", class: "tracklist",
                div { min_height: "{queue_len() * ROW_HEIGHT}px" }

                for idx in start_index()..end_index() {
                    if current_dragging.read().is_some() {
                        if current_dragging().unwrap() > idx && hovering_over() == idx
                            || current_dragging().unwrap() < idx && hovering_over() == idx.max(1) - 1
                        {
                            div { class: "trackitemplaceholder" }
                        }
                    }
                    TrackItem {
                        controller,
                        selected_queue,
                        idx,
                        current_dragging,
                        hovering_over,
                        mouse_y,
                        scroll_y,
                        grab_y,
                        move_queue_item,
                    }
                }
            }
        }

        if queue_editing.read().is_some() {
            QueueOptions { controller, queue_editing, selected_queue }
        }
    }
}

#[component]
pub fn QueueOptions(
    controller: SyncStore<MusicController>,
    queue_editing: Signal<Option<usize>>,
    selected_queue: Signal<usize>,
) -> Element {
    rsx! {
        div { class: "optionsbg", onclick: move |_| queue_editing.set(None),
            div { class: "optionbox", style: "--width: 300px; --height: 100px;",
                h3 { "{controller.queues().get(queue_editing().unwrap()).unwrap().read().queue_type}" }
                button { 
                    onclick: move |_| {
                        controller.remove_queue(queue_editing.unwrap());
                        *selected_queue.write() -= 1;
                        // TODO: if no more queues, add all tracks queue
                    },
                    img { src: REMOVE_ICON }
                    "Remove queue"
                }
                button { onclick: move |_| controller.queue_to_playlist(queue_editing.unwrap()),
                    img { src: EXPORT_ICON }
                    "Save as playlist"
                }
            }
        }
    }
}

#[component]
pub fn TrackItem(
    controller: SyncStore<MusicController>,
    selected_queue: Signal<usize>,
    idx: usize,
    current_dragging: Signal<Option<usize>>,
    hovering_over: Signal<usize>,
    mouse_y: Signal<f32>,
    scroll_y: Signal<f32>,
    grab_y: Signal<f32>,
    move_queue_item: Callback<Event<MouseData>>,
) -> Element {
    const ROW_HEIGHT: usize = 62;

    let is_current_queue = use_memo(move || {
        controller.current_queue_index()() == selected_queue()
    });

    let current_track = use_memo(move || {
        controller.queues().get(selected_queue()).unwrap().read().current_track
    });

    rsx! {
        div {
            class: "trackitem noselect",
            class: if is_current_queue() && current_track() == idx { "current" },
            class: if current_dragging() == Some(idx) { "dragging" },
            position: "absolute",
            top: if current_dragging() == Some(idx) { 
                "{(mouse_y() - grab_y()) + scroll_y() - 68.0}px"
            } else if current_dragging().is_some() && hovering_over() >= idx {
                "{(idx.max(1) - 1) * ROW_HEIGHT}px"
            } else {
                "{idx * ROW_HEIGHT}px"
            },
            onclick: move |_| {
                if current_dragging.read().is_some() {
                    return;
                }
                controller.set_queue_and_track(selected_queue(), idx);
                VIEW.write().current = View::Song;
            },

            img {
                class: "trackbutton draghandle",
                src: DRAG_HANDLE_ICON,
                onmousedown: move |e| {
                    current_dragging.set(Some(idx));
                    grab_y.set(e.data.coordinates().element().y as f32);
                },
                onmouseup: move_queue_item,
                onclick: |e| e.stop_propagation(),
            }

            img {
                class: "trackitemicon",
                loading: "onvisible",
                src: "/trackimage/{controller.queues().get(selected_queue()).unwrap().read().track(idx)}?origin=queue",
            }

            span { "{controller.get_queue_track(selected_queue(), idx).read().title}" }

            div { flex_grow: 1 }

            img {
                class: "trackbutton",
                onclick: move |e| {
                    e.stop_propagation();
                    *TRACKOPTION.write() = Some(
                        controller.queues().get(selected_queue()).unwrap().read().track(idx),
                    );
                },
                src: VERT_ICON,
            }
        }
    }
}
