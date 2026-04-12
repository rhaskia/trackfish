pub mod bulkeditor;
pub mod duplicates;
pub mod tagging;

use dioxus::{prelude::*, stores::SyncStore};
use crate::app::MusicController;

use super::explorer::ExplorerSwitch;

use bulkeditor::BulkEditor;
use duplicates::DuplicateMenu;
use tagging::TaggingMenu;

pub enum LibraryMenu {
    Duplicates,
    Autotagging,
    BulkEditor,
}
    
#[component]
pub fn LibraryManagement(controller: SyncStore<MusicController>) -> Element {
    let mut menu = use_signal(|| LibraryMenu::Duplicates);
    
    rsx!{
        ExplorerSwitch { controller }

        div { class: "librarymanagementview view", id: "librarymanagement",
            div { class: "header",
                button {
                    class: "basicbutton",
                    onclick: move |_| menu.set(LibraryMenu::Duplicates),
                    "Duplicates"
                }
                button {
                    class: "basicbutton",
                    onclick: move |_| menu.set(LibraryMenu::Autotagging),
                    "Autotagging"
                }
                button {
                    class: "basicbutton",
                    onclick: move |_| menu.set(LibraryMenu::BulkEditor),
                    "Bulk Editor"
                }
            }

            match *menu.read() {
                LibraryMenu::Duplicates => rsx! {
                    DuplicateMenu { controller }
                },
                LibraryMenu::Autotagging => rsx! {
                    TaggingMenu { controller }
                },
                LibraryMenu::BulkEditor => rsx! {
                    BulkEditor { controller } 
                }
            }
        }
    }
}