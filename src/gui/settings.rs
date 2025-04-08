use dioxus::prelude::*;
use crate::app::MusicController;
use super::{View, VIEW};
use std::fmt::{Display, Formatter};

#[component]
pub fn Settings(controller: Signal<MusicController>) -> Element {
    let mut settings_menu = use_signal(|| None);

    rsx!{
        div {
            display: if VIEW.read().current != View::Settings { "none" },
            class: "settingsview",
            match settings_menu() {
                Some(menu) => match menu {
                    SettingsMenu::Radio => rsx!{ RadioSettings { controller } },
                    _ => rsx!{}
                }
                None => {
                    rsx!{
                        h2 {
                            class: "settingsheader",
                            "Settings"
                        }
                        button {
                            class: "settingsbutton",
                            onclick: move |_| settings_menu.set(Some(SettingsMenu::Radio)),
                            "Radio Settings"
                        }
                        button {
                            class: "settingsbutton",
                            onclick: move |_| settings_menu.set(Some(SettingsMenu::Audio)),
                            "Audio"
                        }
                        button {
                            class: "settingsbutton",
                            onclick: move |_| settings_menu.set(Some(SettingsMenu::Library)),
                            "Song library"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsHeader(menu: Signal<SettingsMenu>) -> Element {
    rsx!{
        div {
            class: "settingsheader",
            button {
                "Exit"
            }
            "{menu}"
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
            SettingsMenu::Radio => f.write_str("Audio Settings"),
            SettingsMenu::Audio => f.write_str("Radio Settings"),
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
            div {
                span { "Volume" } 
            }
            div {
                display: "flex",
                flex_direction: "column",
                input {
                    r#type: "range",
                    value: "{volume}",
                    oninput: move |e| controller.write().set_volume(e.parsed::<f32>().unwrap() / 100.0)
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
            div {
                span { "Music Directory" }
                br { }
                span { "Radio Temperature" } 
            }
            div {
                display: "flex",
                flex_direction: "column",
                input { 
                    onchange: move |e| controller.write().set_directory(e.value()),
                }
                input {
                    r#type: "range",
                    max: "20.0",
                    value: "10.0",
                    oninput: move |e| controller.write().set_temp(e.parsed::<f32>().unwrap() / 10.0)
                }
            }
        }
    }
}
