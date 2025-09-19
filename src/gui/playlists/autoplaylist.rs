use dioxus::prelude::*;
use crate::app::MusicController;
use crate::app::autoplaylist::Condition;
use crate::app::autoplaylist::{StrIdentifier, NumIdentifier, Identifier};
use crate::gui::icons::*;
use crate::gui::VIEW;

#[component]
pub fn AutoplaylistView(controller: SyncSignal<MusicController>) -> Element {
    let mut explorer_settings = use_signal(|| false);
    let base_path = Vec::new();

    rsx!{
        div { class: "tracksviewheader",
            img {
                onclick: move |_| VIEW.write().autoplaylist = None,
                src: BACK_ICON,
            }

            h3 {
                "{controller.read().autoplaylists[VIEW.write().autoplaylist.unwrap()].name}"
            }

            img { onclick: move |_| explorer_settings.set(true), src: VERT_ICON }

            "{controller.read().autoplaylists[VIEW.write().autoplaylist.unwrap()].conditions:?}"
        }

        div {
            class: "tracksview",

            ConditionView { controller, path: base_path }
        } 
    }
}

#[component]
pub fn ConditionView(controller: SyncSignal<MusicController>, path: Vec<usize>) -> Element {
    rsx!{
        match &controller.read().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path.clone()] {
            Condition::All(conditions) => rsx!{
                div {
                    class: "condition condition-all",
                    h3 { "All" }

                    for i in 0..conditions.len() {
                        ConditionView { controller, path: { let mut p = path.clone(); p.push(i); p } }
                    }
                }
            },
            Condition::Any(conditions) => rsx!{
                div {
                    class: "condition condition-any",
                    h3 { "Any" }

                    for i in 0..conditions.len() {
                        ConditionView { controller, path: { let mut p = path.clone(); p.push(i); p } }
                    }
                }
            },
            Condition::Is(ident, value) => rsx!{
                div {
                    class: "condition condition-is",

                    StrSelect { controller, ident: *ident, path: path.clone() }

                    "IS"

                    input { value: "{value}" }
                }
            },
            Condition::Has(ident, value) => rsx!{
                div { 
                    class: "condition condition-is",

                    StrSelect { controller, ident: *ident, path: path.clone() }

                    "HAS"

                    input { 
                        value: "{value}"
                    }
                }
            },
            Condition::Greater(ident, value) => rsx!{
                div {
                    class: "condition condition-greater",

                    NumSelect { controller, ident: *ident, path: path.clone() }

                    "GREATER THAN"

                    input {
                        value: "{value}"
                    }
                }
            },
            Condition::Lesser(ident, value) => rsx!{
                div {
                    class: "condition condition-lesser",

                    NumSelect { controller, ident: *ident, path: path.clone() }

                    "LESSER THAN"

                    input {
                        value: "{value}"
                    }
                }
            },
            Condition::EqualTo(ident, value) => rsx!{
                div {
                    class: "condition condition-equalto",

                    NumSelect { controller, ident: *ident, path: path.clone() }

                    "EQUAL TO"

                    input {
                        value: "{value}"
                    }
                }
            },
            Condition::Not(cond) => rsx!{
                div {
                    class: "condition condition-not",
                    "NOT",
                    ConditionView { controller, path: { let mut p = path.clone(); p.push(0); p } }
                }
            },
            Condition::Missing(ident) => rsx!{
                div {
                    class: "condition condition-missing",
                    "MISSING",
                    IdentSelect { controller, ident: *ident, path: path.clone() }
                }
            }
        }
    }
}

#[component]
pub fn StrSelect(controller: SyncSignal<MusicController>, ident: StrIdentifier, path: Vec<usize>) -> Element {
    rsx!{
        select { 
            onchange: move |e| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path.clone()].set_ident(e.value()),
            value: "{ident}",
            option { "Title" }
            option { "Artist" }
            option { "Album" }
            option { "Genre" }
        }
    }
}

#[component]
pub fn NumSelect(controller: SyncSignal<MusicController>, ident: NumIdentifier, path: Vec<usize>) -> Element {
    rsx!{
        select { 
            value: "{ident}",
            option { "Year" }
            option { "Length" }
            option { "Energy" }
        }
    }
}

#[component]
pub fn IdentSelect(controller: SyncSignal<MusicController>, ident: Identifier, path: Vec<usize>) -> Element {
    rsx!{
        select { 
            value: "{ident}",
            option { "Title" }
            option { "Artist" }
            option { "Album" }
            option { "Genre" }
            option { "Year" }
            option { "Length" }
            option { "Energy" }
        }
    }
}
