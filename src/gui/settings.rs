use dioxus::prelude::*;
use crate::MusicController;

#[component]
pub fn Settings(controller: Signal<MusicController>) -> Element {
    rsx!{
        div {
            class: "settingsview",
            span { "Music Directory" }
            input { 
                onchange: move |e| controller.write().set_directory(e.value()),
            }
            br { }
            span { "Volume" } 
            input {
                r#type: "range",
                value: "100.0",
                oninput: move |e| controller.write().set_volume(e.parsed::<f32>().unwrap() / 100.0)
            }
            br { }
            span { "Radio Temperature" } 
            input {
                r#type: "range",
                max: "20.0",
                value: "10.0",
                oninput: move |e| controller.write().set_temp(e.parsed::<f32>().unwrap() / 10.0)
            }
        }
    }
}
