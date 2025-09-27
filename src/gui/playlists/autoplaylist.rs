use dioxus::prelude::*;
use crate::app::MusicController;
use crate::app::autoplaylist::Condition;
use crate::app::autoplaylist::{StrIdentifier, NumIdentifier, Identifier};
use crate::gui::icons::*;
use crate::gui::VIEW;
use crate::gui::View;
use crate::gui::TRACKOPTION;

#[component]
pub fn AutoplaylistView(controller: SyncSignal<MusicController>) -> Element {
    let mut explorer_settings = use_signal(|| false);
    let base_path = Vec::new();
    let mut tracks: Signal<Vec<usize>> = use_signal(|| Vec::new());

    rsx!{
        div { class: "tracksviewheader",
            img {
                onclick: move |_| VIEW.write().autoplaylist = None,
                src: BACK_ICON,
            }

            h3 {
                "{controller.read().autoplaylists[VIEW.read().autoplaylist.unwrap()].name}"
            }

            img { onclick: move |_| explorer_settings.set(true), src: VERT_ICON }
        }

        div {
            class: "tracksview",

            ConditionView { controller, path: base_path }
            div {
                class: "autoplaylist-menu",
                button { 
                    onclick: move |_| tracks.set(controller.read().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions.qualify_tracks(&controller.read().all_tracks)),
                    "Refresh"
                }

                button { 
                    onclick: move |_| controller.read().autoplaylists[VIEW.read().autoplaylist.unwrap()].save(),
                    "Save"
                }
            }

            for i in 0..tracks.read().len() {
                div {
                    class: "trackitem",
                    onclick: move |_| {
                        VIEW.write().open(View::Song);
                        controller.write().play_autoplaylist_at(tracks(), VIEW.read().autoplaylist.unwrap(), tracks.read()[i]);
                    },

                    img {
                        class: "trackitemicon",
                        src: "/trackimage/{tracks.read()[i]}",
                        loading: "onvisible",
                    }

                    span { "{controller.read().get_track(tracks.read()[i]).unwrap().title}" }

                    div { flex_grow: 1 }

                    img {
                        class: "trackbutton",
                        loading: "onvisible",
                        onclick: move |e| {
                            e.stop_propagation();
                            *TRACKOPTION.write() = Some(tracks.read()[i]);
                        },
                        src: VERT_ICON,
                    }
                }

            }
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
                    h3 { "All of:" }

                    for i in 0..conditions.len() {
                        div {
                            class: "condition-slot",
                            ConditionView { controller, path: { let mut p = path.clone(); p.push(i); p } }
                            RemoveCondition { controller, path: path.clone(), i }
                        }
                    }

                    AddCondition { controller, path }
                }
            },
            Condition::Any(conditions) => rsx!{
                div {
                    class: "condition condition-any",
                    h3 { "Any of:" }

                    for i in 0..conditions.len() {
                        div {
                            class: "condition-slot",
                            ConditionView { controller, path: { let mut p = path.clone(); p.push(i); p } }
                            RemoveCondition { controller, path: path.clone(), i }
                        }
                    }

                    AddCondition { controller, path }
                }
            },
            Condition::Is(ident, value) => rsx!{
                div {
                    class: "condition condition-is",

                    StrSelect { controller, ident: *ident, path: path.clone() }

                    "IS"

                    ValueInput { controller, ident: *ident, value, path: path.clone() }
                }
            },
            Condition::Has(ident, value) => rsx!{
                div { 
                    class: "condition condition-is",

                    StrSelect { controller, ident: *ident, path: path.clone() }

                    "HAS"

                    ValueInput { controller, ident: *ident, value, path: path.clone() }
                }
            },
            Condition::Greater(ident, value) => rsx!{
                div {
                    class: "condition condition-greater",

                    NumSelect { controller, ident: *ident, path: path.clone() }

                    "GREATER THAN"

                    NumInput { controller, ident: *ident, value, path: path.clone() }
                }
            },
            Condition::Lesser(ident, value) => rsx!{
                div {
                    class: "condition condition-lesser",

                    NumSelect { controller, ident: *ident, path: path.clone() }

                    "LESSER THAN"

                    NumInput { controller, ident: *ident, value, path: path.clone() }
                }
            },
            Condition::EqualTo(ident, value) => rsx!{
                div {
                    class: "condition condition-equalto",

                    NumSelect { controller, ident: *ident, path: path.clone() }

                    "EQUAL TO"

                    NumInput { controller, ident: *ident, value, path: path.clone() }
                }
            },
            Condition::Not(cond) => rsx!{
                div {
                    class: "condition condition-not",
                    "NOT",
                    div {
                        class: "condition-slot",
                        if cond.is_some() {
                            ConditionView { controller, path: { let mut p = path.clone(); p.push(0); p } }
                            RemoveCondition { controller, path: path.clone(), i: 0 }
                        } else {
                            AddCondition { controller, path }
                        }
                    } 
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
pub fn AddCondition(controller: SyncSignal<MusicController>, path: Vec<usize>) -> Element {
    let mut add_condition = move |path: Vec<usize>, cond: Condition| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path].add(cond);
    let path = use_signal(|| path);

    rsx!{
        div {
            class: "condition-add",
            button { 
                onclick: move |_| add_condition(path(), Condition::Is(StrIdentifier::Title, "Title".to_string())),
                "IS"
            }
            button { 
                onclick: move |_| add_condition(path(), Condition::Has(StrIdentifier::Title, "Title".to_string())),
                "HAS"
            }
            button { 
                onclick: move |_| add_condition(path(), Condition::Greater(NumIdentifier::Year, 2000)),
                "GREATER"
            }
            button { 
                onclick: move |_| add_condition(path(), Condition::Lesser(NumIdentifier::Year, 2000)),
                "LESSER"
            }
            button { 
                onclick: move |_| add_condition(path(), Condition::EqualTo(NumIdentifier::Year, 2000)),
                "EQUAL TO"
            }
            button { 
                onclick: move |_| add_condition(path(), Condition::All(Vec::new())),
                "ALL"
            }
            button { 
                onclick: move |_| add_condition(path(), Condition::Any(Vec::new())),
                "ANY"
            }
            button { 
                onclick: move |_| add_condition(path(), Condition::Not(None)),
                "NOT"
            }
            button { 
                onclick: move |_| add_condition(path(), Condition::Missing(Identifier::Str(StrIdentifier::Title))),
                "MISSING"
            }
        }
    }
}

#[component]
pub fn RemoveCondition(controller: SyncSignal<MusicController>, path: Vec<usize>, i: usize) -> Element {
    rsx!{
        button {
            class: "svg-button",
            onclick: move |_| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path.clone()].remove(i),
            style: "background: url({crate::gui::icons::DELETE_ICON})",
            width: "24px",
            height: "24px",
            margin: "4px 4px 0 0",
            border: "none",
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
            onchange: move |e| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path.clone()].set_ident(e.value()),
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
            onchange: move |e| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path.clone()].set_ident(e.value()),
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

#[component]
pub fn ValueInput(controller: SyncSignal<MusicController>, ident: StrIdentifier, value: String, path: Vec<usize>) -> Element {
    rsx!{
        input {
            oninput: move |e| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path.clone()].set_value(e.value()),
            value: "{value}"
        }
    }
}

#[component]
pub fn NumInput(controller: SyncSignal<MusicController>, ident: NumIdentifier, value: String, path: Vec<usize>) -> Element {
    rsx!{
        input {
            r#type: "number",
            oninput: move |e| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path.clone()].set_value(e.value()),
            value: "{value}"
        }
    }
}
