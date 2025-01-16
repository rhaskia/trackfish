use dioxus::prelude::*;
use crate::app::MusicController;

#[component]
pub fn AllTracks(controller: Signal<MusicController>) -> Element {
    rsx!{
        div {
            class: "tracklist",
            for i in 0..controller.read().all_tracks.len() {
                div {
                    class: "trackitem",
                    id: "trackitem-{i}",
                    onclick: move |_| controller.write().add_all_queue(i),
                    img { src: "/trackimage/{i}" }
                    span { "{controller.read().all_tracks[i].title}" }
                    div { flex_grow: 1, }
                    img { src: "/assets/vert.svg" }
                } 
            }
        }    
    }
}
