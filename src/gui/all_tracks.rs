use dioxus::prelude::*;
use crate::queue::QueueManager;

#[component]
pub fn AllTracks(queue: Signal<QueueManager>) -> Element {
    rsx!{
        div {
            class: "tracklist",
            for i in 0..queue.read().all_tracks.len() {
                div {
                    class: "trackitem",
                    onclick: move |_| queue.write().add_all_queue(i),
                    img { src: "/trackimage/{i}" }
                    span { "{queue.read().all_tracks[i].title}" }
                    div { flex_grow: 1, }
                    img { src: "/assets/vert.svg" }
                } 
            }
        }    
    }
}
