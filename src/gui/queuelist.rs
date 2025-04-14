use dioxus::prelude::*;
use super::{View, VIEW};
use crate::app::MusicController;

#[component]
pub fn QueueList(controller: Signal<MusicController>) -> Element {
    let mut selected_queue = use_signal(|| 0);

    use_effect(move || {
        selected_queue.set(controller.read().current_queue);
    });

    rsx! {
        div { id: "queuelist", class: "tracklist",
            display: if VIEW.read().current != View::Queue { "none" },

            div {
                class: "selectwrapper",
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
            span {
                margin: "2px 10px",
                "Track: {controller.read().current_queue().current_track + 1}/{controller.read().current_queue().len()}"
            }
            for idx in 0..controller.read().get_queue(selected_queue()).cached_order.len() {
                TrackItem { controller, selected_queue, idx }
            }
        }
    }
}

#[component]
pub fn TrackItem(controller: Signal<MusicController>, selected_queue: Signal<usize>, idx: usize) -> Element {
    let title = use_memo(move || {
        match controller.read().get_track(controller.read().get_queue(selected_queue()).track(idx)) {
            Some(track) => track.title.clone(),
            None => String::new(),
        }
    });

    let is_current = use_memo(move || {
        controller.read().get_queue(selected_queue()).current_track == idx &&
        controller.read().current_queue == selected_queue()
    });

    rsx!{
        div {
            class: "trackitem",
            draggable: true,
            class: if is_current() { "current" },
            onclick: move |_| {
                controller.write().set_queue_and_track(selected_queue(), idx);
                VIEW.write().current = View::Song;
            },
            img { class: "trackbutton draghandle", src: "/assets/icons/draghandle.svg" }
            img { class: "trackitemicon", loading: "onvisible", src: "/trackimage/{controller.read().get_queue(selected_queue()).track(idx)}" }
            span { "{title}" }
            div { flex_grow: 1 }
            img { class: "trackbutton", src: "/assets/icons/vert.svg" }
        }
    }
}
