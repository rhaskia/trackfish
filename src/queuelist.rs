use dioxus::prelude::*;
use crate::queue::QueueManager;

#[component]
pub fn QueueList(queue: Signal<QueueManager>) -> Element {
    let mut selected_queue = use_signal(|| queue.read().current_queue);

    rsx! {
        div {
            class: "queuelist",
            "{selected_queue()}"
            select {
                value: selected_queue(),
                onchange: move |e| selected_queue.set(e.value().parse::<usize>().unwrap()),
                for i in 0..queue.read().queues.len() {
                    option {
                        value: "{i}",
                        "{queue.read().queues[i].queue_type}",
                    }
                }
            }
            for idx in 0..queue.read().get_queue(selected_queue()).cached_order.len() {
                TrackItem { queue, selected_queue, idx } 
            } 
        }
    }
}

#[component]
pub fn TrackItem(queue: Signal<QueueManager>, selected_queue: Signal<usize>, idx: usize) -> Element {
    rsx!{
        div {
            class: "trackitem",
            class: if queue.read().get_queue(selected_queue()).current_track == idx
                   && queue.read().current_queue == selected_queue() { "current" },
            onclick: move |_| queue.write().set_queue_and_track(selected_queue(), idx),
            img { src: "/trackimage/{queue.read().get_queue(selected_queue()).track(idx)}" },
            span {
                "{queue.read().get_track(queue.read().get_queue(selected_queue()).track(idx)).unwrap().title}"
            }
        }
    }
}
