use super::CONTROLLER;
use super::{View, VIEW};
use crate::app::MusicController;
use dioxus::document::eval;
use dioxus::prelude::*;
use std::time::Duration;
use log::info;

#[component]
pub fn QueueList() -> Element {
    let mut selected_queue = use_signal(|| 0);
    let mut current_dragging = use_signal(|| None);
    let mut mouse_y = use_signal(|| 0);
    let mut grab_y = use_signal(|| 0);
    let mut hovering_over = use_signal(|| 0);

    use_effect(move || {
        selected_queue.set(CONTROLLER.read().current_queue);
    });

    use_future(move || async move {
        let mut js = eval(
            r#"
            document.addEventListener('mousemove', function(event) {
                console.log(event.clientY);
                dioxus.send(event.clientY);
            });
        "#,
        );

        loop {
            let position = js.recv().await;
            if let Ok(pos) = position {
                mouse_y.set(pos);
                hovering_over.set(((pos as f32 - 35.0 - 31.0) / 62.0).floor() as usize);
            }
        }
    });

    let move_queue_item = move |e: Event<MouseData>| {
        if let Some(current) = current_dragging() {
            CONTROLLER.write().queues[selected_queue()].swap(current, hovering_over())
        }
        current_dragging.set(None);
    };

    rsx! {
        div { id: "queuelist", class: "tracklist",
            display: if VIEW.read().current != View::Queue { "none" },
            onclick: move_queue_item.clone(),

            div {
                class: "selectwrapper queueselectwrapper",
                select {
                    class: "queueselect",
                    onchange: move |e| selected_queue.set(e.value().parse::<usize>().unwrap()),
                    for i in 0..CONTROLLER.read().queues.len() {
                        option { value: "{i}", selected: i == selected_queue(),
                            "{CONTROLLER.read().queues[i].queue_type}"
                        }
                    }
                }
            }
            span {
                margin: "2px 10px",
                "Track: {CONTROLLER.read().current_queue().current_track + 1}/{CONTROLLER.read().current_queue().len()}"
            }
            for idx in 0..CONTROLLER.read().get_queue(selected_queue()).cached_order.len() {
                if current_dragging.read().is_some() {
                    if current_dragging().unwrap() > idx && hovering_over() == idx ||
                        current_dragging().unwrap() < idx && hovering_over() == idx.max(1) - 1{
                        div { 
                            class: "trackitemplaceholder",
                        }
                    }
                }
                TrackItem { selected_queue, idx, current_dragging, mouse_y, grab_y, move_queue_item }
            }
        }
    }
}

#[component]
pub fn TrackItem(
    selected_queue: Signal<usize>,
    idx: usize,
    current_dragging: Signal<Option<usize>>,
    mouse_y: Signal<i32>,
    grab_y: Signal<i32>,
    move_queue_item: Callback<Event<MouseData>>
) -> Element {
    let title = use_memo(move || {
        match CONTROLLER.read().get_track(CONTROLLER.read().get_queue(selected_queue()).track(idx))
        {
            Some(track) => track.title.clone(),
            None => String::new(),
        }
    });

    let is_current = use_memo(move || {
        CONTROLLER.read().get_queue(selected_queue()).current_track == idx
            && CONTROLLER.read().current_queue == selected_queue()
    });

    rsx! {
        div {
            class: "trackitem noselect",
            class: if is_current() { "current" },
            class: if current_dragging() == Some(idx) { "dragging" },
            top: if current_dragging() == Some(idx) { "calc({mouse_y}px - {grab_y}px - 6px)" },
            onclick: move |_| {
                if current_dragging.read().is_some() { return; }
                CONTROLLER.write().set_queue_and_track(selected_queue(), idx);
                VIEW.write().current = View::Song;
            },
            div {
                class: "trackbutton draghandle",
                background_image: "url(/assets/icons/draghandle.svg)",
                onmousedown: move |e| {
                    current_dragging.set(Some(idx));
                    grab_y.set(e.data.coordinates().element().y as i32);
                },
                onmouseup: move_queue_item,
                onclick: |e| e.stop_propagation(),
            }
            img { class: "trackitemicon", loading: "onvisible", src: "/trackimage/{CONTROLLER.read().get_queue(selected_queue()).track(idx)}" }
            span { "{title}" }
            div { flex_grow: 1 }
            img {
                class: "trackbutton",
                onclick: move |e| {
                    e.stop_propagation();
                    super::TRACKOPTION.set(Some(CONTROLLER.read().get_queue(selected_queue()).track(idx)));
                },
                src: "/assets/icons/vert.svg"
            }
        }
    }
}
