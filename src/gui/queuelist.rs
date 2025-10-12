use super::icons::*;
use super::{View, TRACKOPTION, VIEW};
use crate::app::MusicController;
use dioxus::document::eval;
use dioxus::prelude::*;
use std::time::Duration;
use log::info;

#[component]
pub fn QueueList(controller: SyncSignal<MusicController>) -> Element {
    let mut selected_queue = use_signal(|| 0);
    let mut current_dragging = use_signal(|| None);
    let mut mouse_y = use_signal(|| 0.0);
    let mut scroll_y = use_signal(|| 0.0);
    let grab_y = use_signal(|| 0.0);
    let mut hovering_over = use_signal(|| 0);
    let mut queue_height = use_signal(|| 0.0);
    let mut queue_editing = use_signal(|| None);

    use_effect(move || {
        selected_queue.set(controller.read().current_queue);
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
                    .set((((mouse_y() + scroll_y()) - 35.0 - 31.0) / 62.0).floor() as usize);
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
                    .set((((mouse_y() + scroll_y()) - 35.0 - 31.0) / 62.0).floor() as usize);
            }
        }
    });

    // Callback for moving item into other place in queue
    let move_queue_item = move |_: Event<MouseData>| {
        if let Some(current) = current_dragging() {
            controller.write().queues[selected_queue()].swap(current, hovering_over())
        }
        current_dragging.set(None);
    };

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
                        for i in 0..controller.read().queues.len() {
                            option { value: "{i}", selected: i == selected_queue(),
                                "{controller.read().queues[i].queue_type}"
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
                "Track: {controller.read().current_queue().current_track + 1}/{controller.read().current_queue().len()}"
            }

            // Track items in selected queue
            div { id: "queuelist", class: "tracklist",
                for idx in 0..controller.read().get_queue(selected_queue()).cached_order.len().min(10) {
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
                        mouse_y,
                        grab_y,
                        move_queue_item,
                    }
                }
            }
        }

        if queue_editing.read().is_some() {
            QueueOptions { controller, queue_editing }
        }
    }
}

#[component]
pub fn QueueOptions(
    controller: SyncSignal<MusicController>,
    queue_editing: Signal<Option<usize>>,
) -> Element {
    rsx! {
        div { class: "optionsbg", onclick: move |_| queue_editing.set(None),
            div { class: "optionbox", style: "--width: 300px; --height: 100px;",
                h3 { "{controller.read().queues[queue_editing().unwrap()].queue_type}" }
                button { onclick: move |_| controller.write().remove_queue(queue_editing.unwrap()),
                    img { src: REMOVE_ICON }
                    "Remove queue"
                }
                button { onclick: move |_| controller.write().queue_to_playlist(queue_editing.unwrap()),
                    img { src: EXPORT_ICON }
                    "Save as playlist"
                }
            }
        }
    }
}

#[component]
pub fn TrackItem(
    controller: SyncSignal<MusicController>,
    selected_queue: Signal<usize>,
    idx: usize,
    current_dragging: Signal<Option<usize>>,
    mouse_y: Signal<f32>,
    grab_y: Signal<f32>,
    move_queue_item: Callback<Event<MouseData>>,
) -> Element {
    let title = use_memo(move || {
        match controller
            .read()
            .get_track(controller.read().get_queue(selected_queue()).track(idx))
        {
            Some(track) => track.title.clone(),
            None => String::new(),
        }
    });

    let is_current = use_memo(move || {
        controller.read().get_queue(selected_queue()).current_track == idx
            && controller.read().current_queue == selected_queue()
    });

    rsx! {
        div {
            class: "trackitem noselect",
            class: if is_current() { "current" },
            class: if current_dragging() == Some(idx) { "dragging" },
            top: if current_dragging() == Some(idx) { "calc({(mouse_y() - grab_y())}px - 6px)" },
            onclick: move |_| {
                if current_dragging.read().is_some() {
                    return;
                }
                controller.write().set_queue_and_track(selected_queue(), idx);
                VIEW.write().current = View::Song;
            },

            img {
                class: "trackbutton draghandle",
                src: DRAG_HANDLE_ICON,
                onmousedown: move |e| {
                    current_dragging.set(Some(idx));
                    grab_y.set(e.data.coordinates().element().y as f32);
                    info!("woah");
                },
                onmouseup: move_queue_item,
                onclick: |e| e.stop_propagation(),
            }

            img {
                class: "trackitemicon",
                loading: "onvisible",
                src: "/trackimage/{controller.read().get_queue(selected_queue()).track(idx)}",
            }

            span { "{title}" }

            div { flex_grow: 1 }

            img {
                class: "trackbutton",
                onclick: move |e| {
                    e.stop_propagation();
                    *TRACKOPTION.write() = Some(
                        controller.read().get_queue(selected_queue()).track(idx),
                    );
                },
                src: VERT_ICON,
            }
        }
    }
}
