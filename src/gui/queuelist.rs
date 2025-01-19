use dioxus::prelude::*;
use crate::{View, VIEW};
use crate::app::MusicController;

#[component]
pub fn QueueList(controller: Signal<MusicController>) -> Element {
    let mut selected_queue = use_signal(|| controller.read().current_queue);

    rsx! {
        div { id: "queuelist", class: "tracklist",
            select { onchange: move |e| selected_queue.set(e.value().parse::<usize>().unwrap()),
                for i in 0..controller.read().queues.len() {
                    option { value: "{i}", selected: i == selected_queue(),
                        "{controller.read().queues[i].queue_type}"
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
    rsx!{
        div {
            class: "trackitem",
            class: if controller.read().get_queue(selected_queue()).current_track == idx
    && controller.read().current_queue == selected_queue() { "current" },
            onclick: move |_| {
                controller.write().set_queue_and_track(selected_queue(), idx);
                VIEW.write().current = View::Song;
            },
            img { class: "trackbutton draghandle", src: "/assets/draghandle.svg" }
            img { src: "/trackimage/{controller.read().get_queue(selected_queue()).track(idx)}" }
            span {
                "{controller.read().get_track(controller.read().get_queue(selected_queue()).track(idx)).unwrap().title}"
            }
            div { flex_grow: 1 }
            img { class: "trackbutton", src: "/assets/vert.svg" }
        }
    }
}
