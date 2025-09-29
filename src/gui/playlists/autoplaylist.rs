use dioxus::prelude::*;
use crate::app::MusicController;
use crate::app::autoplaylist::Condition;
use crate::app::autoplaylist::{StrIdentifier, NumIdentifier, Identifier, StrOperator, NumOperator, TimeIdentifier};
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
    let path = use_signal(|| path);
    rsx!{
        match &controller.read().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path()] {
            Condition::All(conditions) => rsx!{
                div {
                    class: "condition condition-all",
                    div {
                        class: "condition-group-toggle",
                        button {
                            class: "current-conditon-toggle toggle-left",
                            "All"
                        }

                        button {
                            class: "toggle-right",
                            onclick: move |_| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path()].toggle_group(),
                            "Any"
                        }

                        AddCondition { controller, path: path() }

                        if !path.read().is_empty() {
                            RemoveCondition { controller, path: { let mut path = path(); path.pop(); path }, i: path().pop().unwrap() }
                        }
                    }

                    for i in 0..conditions.len() {
                        div {
                            class: "condition-slot",
                            ConditionView { controller, path: { let mut p = path(); p.push(i); p } }
                            if !conditions[i].is_all_or_any() {
                                RemoveCondition { controller, path: path(), i }
                            }
                        }
                    }

                }
            },
            Condition::Any(conditions) => rsx!{
                div {
                    class: "condition condition-any",
                    div {
                        class: "condition-group-toggle",
                        button {
                            class: "toggle-left",
                            onclick: move |_| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path()].toggle_group(),
                            "All"
                        }

                        button {
                            class: "current-conditon-toggle toggle-right",
                            "Any"
                        }

                        AddCondition { controller, path: path() }

                        if !path.read().is_empty() {
                            RemoveCondition { controller, path: { let mut path = path(); path.pop(); path }, i: path().pop().unwrap() }
                        }
                    }

                    for i in 0..conditions.len() {
                        div {
                            class: "condition-slot",
                            ConditionView { controller, path: { let mut p = path(); p.push(i); p } }
                            if !conditions[i].is_all_or_any() {
                                RemoveCondition { controller, path: path(), i }
                            }
                        }
                    }

                }
            },
            Condition::StrCondition(ident, op, value) => rsx!{
                IdentSelect { controller, ident: Identifier::Str(*ident), path: path() }

                select {
                    onchange: move |e| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path()].set_op(e.value()),
                    value: "{op}",
                    option { "Is" }
                    option { "Has" }
                    option { "IsNot" }
                    option { "HasNot" }
                    option { "Missing" }
                }

                ValueInput { controller, ident: *ident, value, path: path() }
            },
            Condition::NumCondition(ident, op, value) => rsx!{
                IdentSelect { controller, ident: Identifier::Num(*ident), path: path() }

                select {
                    onchange: move |e| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path()].set_op(e.value()),
                    value: "{op}",
                    option { "Greater" }
                    option { "Lesser" }
                    option { "Equals" }
                    option { "NotEqual" }
                    option { "Missing" }
                }

                NumInput { controller, ident: *ident, value, path: path() }
            },
            Condition::TimeCondition(ident, op, value) => rsx!{
                IdentSelect { controller, ident: Identifier::Time(*ident), path: path() }

                select {
                    onchange: move |e| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path()].set_op(e.value()),
                    value: "{op}",
                    option { "Greater" }
                    option { "Lesser" }
                    option { "Equals" }
                    option { "NotEqual" }
                    option { "Missing" }
                }

                input {
                    r#type: "number",
                }
                
                select {
                    option { "Seconds" }
                    option { "Minutes" }
                    option { "Hours" }
                    option { "Days" }
                    option { "Weeks" }
                    option { "Months" }
                    option { "Years" }
                }
            },
        }
    }
}

#[component]
pub fn AddCondition(controller: SyncSignal<MusicController>, path: Vec<usize>) -> Element {
    let path = use_signal(|| path);
    let mut add_condition = move |cond: Condition| controller.write().autoplaylists[VIEW.read().autoplaylist.unwrap()].conditions[path()].add(cond);

    rsx!{
        button {
            class: "condition-add",
            onclick: move |_| add_condition(Condition::All(vec![])),
            img { src: ADD_ICON }
            "Group"
        }
        button { 
            class: "condition-add",
            onclick: move |_| add_condition(Condition::StrCondition(StrIdentifier::Title, StrOperator::Is, String::new())),
            img { src: ADD_ICON }
            "Rule"
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
            margin: "4px",
            border: "none",
            filter: "sepia(1) hue-rotate(-45deg) saturate(5)",
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


