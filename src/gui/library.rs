use dioxus::{prelude::*, stores::SyncStore};
use crate::app::MusicController;

#[component]
pub fn LibraryManagement(controller: SyncStore<MusicController>) -> Element {
    rsx!{
        div { class: "librarymanagementview" }
    }
}