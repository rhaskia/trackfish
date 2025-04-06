use dioxus::prelude::*;
use crate::app::MusicController;
use super::{View, VIEW};

#[component]
pub fn Settings(controller: Signal<MusicController>) -> Element {
    let volume = controller.read().settings.volume * 100.0;

    rsx!{
        div {
            display: if VIEW.read().current != View::Settings { "none" },
            class: "settingsview",
            h2 { class: "settingsheader", "Settings" }
            div {
                span { "Music Directory" }
                br { }
                span { "Volume" } 
                br { }
                span { "Radio Temperature" } 
            }
            div {
                display: "flex",
                flex_direction: "column",
                input { 
                    onchange: move |e| controller.write().set_directory(e.value()),
                }
                input {
                    r#type: "range",
                    value: "{volume}",
                    oninput: move |e| controller.write().set_volume(e.parsed::<f32>().unwrap() / 100.0)
                }
                input {
                    r#type: "range",
                    max: "20.0",
                    value: "10.0",
                    oninput: move |e| controller.write().set_temp(e.parsed::<f32>().unwrap() / 10.0)
                }
            }
        }
    }
}
