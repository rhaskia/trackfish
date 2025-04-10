use dioxus::prelude::*;
use crate::app::MusicController;
use super::{View, VIEW};
use std::fmt::{Display, Formatter};
use log::info;

#[component]
pub fn Settings(controller: Signal<MusicController>) -> Element {
    let mut settings_menu = use_signal(|| SettingsMenu::Audio);
    let mut extended_list = use_signal(|| false);

    let mut set_menu = move |menu: SettingsMenu| {
        settings_menu.set(menu);
        extended_list.set(false);
    };

    rsx!{
        div {
            display: if VIEW.read().current != View::Settings { "none" },
            class: "settingsview",
            button {
                class: "settingslistbutton",
                onclick: move |_| extended_list.set(!extended_list()),
            }
            div {
                class: "settingslist", 
                class: if !extended_list() { "closed" },
                button {
                    class: "settingsbutton",
                    onclick: move |_| set_menu(SettingsMenu::Audio),
                    "Audio"
                }
                button {
                    class: "settingsbutton",
                    onclick: move |_| set_menu(SettingsMenu::Radio),
                    "Radio Settings"
                }
                button {
                    class: "settingsbutton",
                    onclick: move |_| set_menu(SettingsMenu::Library),
                    "Song library"
                }
            }
            match settings_menu() {
                SettingsMenu::Radio => rsx!{ RadioSettings { controller } },
                SettingsMenu::Library => rsx!{ LibrarySettings { controller } },
                SettingsMenu::Audio => rsx!{ AudioSettings { controller } },
            }
        }
    }
}

#[derive(Clone, Debug)]
enum SettingsMenu {
    Radio,
    Audio,
    Library
}

impl Display for SettingsMenu {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SettingsMenu::Radio => f.write_str("Radio Settings"),
            SettingsMenu::Audio => f.write_str("Audio Settings"),
            SettingsMenu::Library => f.write_str("Track Library"),
        }
    }
}

#[component]
fn AudioSettings(controller: Signal<MusicController>) -> Element {
    let volume = controller.read().settings.volume;

    rsx!{
        div {
            class: "settingsmenu",
            h2 { class: "settingsbar", "Audio" }
            div {
                class: "settingbox",
                span { "Volume" } 
                input {
                    r#type: "range",
                    max: "1",
                    step: "0.01",
                    value: "{controller.read().settings.volume}",
                    oninput: move |e| controller.write().set_volume(e.parsed::<f32>().unwrap())
                }
            }
        }
    }
}

#[component]
fn RadioSettings(controller: Signal<MusicController>) -> Element {
    rsx!{
        div {
            class: "settingsmenu",
            h2 { class: "settingsbar", "Radio" }
            div {
                class: "settingbox",
                span {
                    "Radio Temperature"
                }
                div {
                    class: "settingsinput",
                    input {
                        r#type: "range",
                        max: "2.0",
                        step: "0.01",
                        value: "{controller.read().settings.radio.temp}",
                        oninput: move |e| controller.write().set_temp(e.parsed::<f32>().unwrap())
                    }
                    input {
                        class: "smalltextinput",
                        r#type: "text",
                        value: "{controller.read().settings.radio.temp}",
                    }
                }
            }
            div {
                class: "settingbox",
                span { "Track features to use" }
                select {
                    onchange: |e| info!("{e:?}"),
                    option {
                        "First"
                    }
                    option {
                        "Last"
                    }
                    option {
                        "Average"
                    }
                }
            }
        }
    }
}

#[component]
fn LibrarySettings(controller: Signal<MusicController>) -> Element {
    rsx!{
        div {
            class: "settingsmenu",
            h2 { class: "settingsbar", "Library" }
            div {
                class: "settingbox",
                span { "Music Directory" }
                input { 
                    r#type: "text",
                    value: "{controller.write().settings.directory}",
                    onchange: move |e| controller.write().set_directory(e.value()),
                }
            }
        }
    }
}
