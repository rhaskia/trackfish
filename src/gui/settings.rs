use dioxus::prelude::*;
use crate::app::MusicController;
use super::{View, VIEW};
use std::fmt::{Display, Formatter};
use log::info;
use super::CONTROLLER;

#[component]
pub fn Settings() -> Element {
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
                    img { src: "assets/icons/audio.svg" }
                    "Audio"
                }
                button {
                    class: "settingsbutton",
                    onclick: move |_| set_menu(SettingsMenu::Radio),
                    img { src: "assets/icons/radio.svg" }
                    "Radio Settings"
                }
                button {
                    class: "settingsbutton",
                    onclick: move |_| set_menu(SettingsMenu::Library),
                    img { src: "assets/icons/library.svg" }
                    "Song library"
                }
            }
            match settings_menu() {
                SettingsMenu::Radio => rsx!{ RadioSettings { } },
                SettingsMenu::Library => rsx!{ LibrarySettings { } },
                SettingsMenu::Audio => rsx!{ AudioSettings { } },
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
fn AudioSettings() -> Element {
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
                    value: "{CONTROLLER.read().settings.volume}",
                    oninput: move |e| CONTROLLER.write().set_volume(e.parsed::<f32>().unwrap())
                }
            }
        }
    }
}

#[component]
fn RadioSettings() -> Element {
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
                        value: "{CONTROLLER.read().settings.radio.temp}",
                        oninput: move |e| CONTROLLER.write().set_temp(e.parsed::<f32>().unwrap())
                    }
                    input {
                        class: "smalltextinput",
                        r#type: "text",
                        value: "{CONTROLLER.read().settings.radio.temp}",
                    }
                }
            }
            div {
                class: "settingbox",
                span { "Track features to use" }
                div {
                    class: "selectwrapper",
                    select {
                        class: "settingsselect",
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
            div {
                class: "settingbox",
                span { "Same artist penalty" }
                div {
                    class: "settingsinput",
                    input {
                        r#type: "range",
                        max: "1.0",
                        step: "0.01",
                        value: "{CONTROLLER.read().settings.radio.artist_penalty}",
                        oninput: move |e| CONTROLLER.write().settings.radio.artist_penalty = e.parsed::<f32>().unwrap(),
                    }
                    input {
                        class: "smalltextinput",
                        r#type: "text",
                        value: "{CONTROLLER.read().settings.radio.artist_penalty}",
                    }
                }
            }
            div {
                class: "settingbox",
                span { "Same album penalty" }
                div {
                    class: "settingsinput",
                    input {
                        r#type: "range",
                        max: "1.0",
                        step: "0.01",
                        value: "{CONTROLLER.read().settings.radio.album_penalty}",
                        oninput: move |e| CONTROLLER.write().settings.radio.album_penalty = e.parsed::<f32>().unwrap(),
                    }
                    input {
                        class: "smalltextinput",
                        r#type: "text",
                        value: "{CONTROLLER.read().settings.radio.album_penalty}",
                    }
                }
            }
        }
    }
}

#[component]
fn LibrarySettings() -> Element {
    rsx!{
        div {
            class: "settingsmenu",
            h2 { class: "settingsbar", "Library" }
            div {
                class: "settingbox",
                span { "Music Directory" }
                input { 
                    r#type: "text",
                    value: "{CONTROLLER.read().settings.directory}",
                    onchange: move |e| CONTROLLER.write().set_directory(e.value()),
                }
            }
        }
    }
}
