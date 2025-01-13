use dioxus::prelude::*;
use crate::queue::QueueManager;

#[component]
pub fn AllTracks(queue: Signal<QueueManager>) -> Element {
    rsx!{
        div {
            class: "alltrackslist",
            for i in 0..queue.read().all_tracks.len() {
                div {
                    class: "trackitem",
                    img { src: "/trackimage/{i}" }
                    span { "{queue.read().all_tracks[i].title}" }
                } 
            }
        }    
    }
}
