use dioxus::prelude::*;
use crate::app::MusicController;
use super::LIBRARY_MANAGEMENT_OPEN;

pub enum LibraryMenu {
    Duplicates,
    Autotagging
}

#[component]
pub fn LibraryManagement(controller: SyncSignal<MusicController>) -> Element {
    let mut menu = use_signal(|| LibraryMenu::Duplicates);
    
    rsx!{
        div { class: "librarymanagementview view", id: "librarymanagement",
            div { class: "header",
                button { onclick: move |_| menu.set(LibraryMenu::Duplicates), "Duplicates" }
                button { onclick: move |_| menu.set(LibraryMenu::Autotagging), "Autotagging" }
            }

            match *menu.read() {
                LibraryMenu::Duplicates => rsx! {
                    DuplicateMenu { controller }
                },
                LibraryMenu::Autotagging => rsx! {
                    TaggingMenu { controller }
                },
            }
        }
    }
}

#[component]
pub fn DuplicateMenu(controller: SyncSignal<MusicController>) -> Element {
    let mut duplicates: Signal<Vec<Vec<usize>>> = use_signal(|| Vec::new());

    rsx!{
        button { onclick: move |_| duplicates.set(controller.read().find_duplicates()),
            "Load duplicates"
        }
        for i in 0..duplicates.read().len() {
            "{duplicates.read()[i]:?}"
        }
    }
}

#[component]
pub fn TaggingMenu(controller: SyncSignal<MusicController>) -> Element {
    rsx!{}
}