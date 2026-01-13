use dioxus::prelude::*;
use crate::app::MusicController;

#[component]
pub fn LibraryManagement(controller: SyncSignal<MusicController>) -> Element {
    rsx!{
        div { class: "librarymanagementview" }
    }
}