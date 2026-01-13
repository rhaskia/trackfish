use super::icons::*;
use crate::app::MusicController;
use dioxus::prelude::*;
use log::info;
use std::fmt::{Display, Formatter};

#[component]
pub fn Settings(controller: SyncSignal<MusicController>) -> Element {
    let mut settings_menu = use_signal(|| SettingsMenu::Audio);
    let mut extended_list = use_signal(|| false);

    let mut set_menu = move |menu: SettingsMenu| {
        settings_menu.set(menu);
        extended_list.set(false);
    };

    rsx! {
        div { class: "settingsview view", id: "settingsview",

            div { class: "settingslist", class: if !extended_list() { "closed" },
                button {
                    class: "settingslistbutton",
                    top: if cfg!(target_os = "android") { "calc(10px + 30pt)" },
                    background: "url({MENU_ICON})",
                    onclick: move |_| extended_list.set(!extended_list()),
                }
                button {
                    class: "settingsbutton",
                    onclick: move |_| set_menu(SettingsMenu::Audio),
                    img { src: AUDIO_ICON }
                    "Audio Settings"
                }
                button {
                    class: "settingsbutton",
                    onclick: move |_| set_menu(SettingsMenu::Radio),
                    img { src: RADIO_ICON }
                    "Radio Settings"
                }
                button {
                    class: "settingsbutton",
                    onclick: move |_| set_menu(SettingsMenu::Library),
                    img { src: LIBRARY_ICON }
                    "Song library"
                }
                button {
                    class: "settingsbutton",
                    onclick: move |_| set_menu(SettingsMenu::Ui),
                    img { src: PALETTE_ICON }
                    "UI Settings"
                }
            }

            match settings_menu() {
                SettingsMenu::Radio => rsx! {
                    RadioSettings { controller }
                },
                SettingsMenu::Library => rsx! {
                    LibrarySettings { controller }
                },
                SettingsMenu::Audio => rsx! {
                    AudioSettings { controller }
                },
                SettingsMenu::Ui => rsx! {
                    UiSettings { controller }
                },
            }
        }
    }
}

#[derive(Clone, Debug)]
enum SettingsMenu {
    Radio,
    Audio,
    Library,
    Ui,
}

impl Display for SettingsMenu {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SettingsMenu::Radio => f.write_str("Radio Settings"),
            SettingsMenu::Audio => f.write_str("Audio Settings"),
            SettingsMenu::Library => f.write_str("Track Library"),
            SettingsMenu::Ui => f.write_str("Ui Settings"),
        }
    }
}

#[component]
fn AudioSettings(controller: SyncSignal<MusicController>) -> Element {
    rsx! {
        div { class: "settingsmenu",
            h2 { class: "settingsbar", "Audio" }
            div { class: "settingbox",
                SettingsInput {
                    label: "Volume",
                    max: "1",
                    value: "{controller.read().settings.volume}",
                    oninput: move |e: Event<FormData>| controller.write().set_volume(e.parsed::<f32>().unwrap()),
                }
            }
        }
    }
}

#[component]
fn UiSettings(controller: SyncSignal<MusicController>) -> Element {
    rsx! {
        div { class: "settingsmenu",
            h2 { class: "settingsbar", "UI" }
            div { class: "settingbox",
                span { "Keep explorer navigation to explorer menu?" }
                input {
                    r#type: "checkbox",
                    value: "{controller.read().settings.ui.hide_explorer_buttons}",
                    oninput: move |value| {
                        controller.write().settings.ui.hide_explorer_buttons = value.value() == "true";
                    },
                }
            }
        }
    }
}

#[component]
fn RadioSettings(controller: SyncSignal<MusicController>) -> Element {
    rsx! {
        form {
            class: "settingsmenu",
            onchange: move |_| controller.write().settings.save(),
            h2 { class: "settingsbar", "Radio" }
            SettingsInput {
                label: "Radio Temperature",
                max: "2.0",
                oninput: move |e: Event<FormData>| controller.write().set_temp(e.parsed::<f32>().unwrap()),
                value: "{controller.read().settings.radio.temp}",
            }

            div { class: "settingbox",
                span { "Track features to use" }
                div { class: "selectwrapper",
                    select { class: "settingsselect", onchange: |e| info!("{e:?}"),
                        option { "First" }
                        option { "Last" }
                        option { "Average" }
                    }
                }
            }

            SettingsInput {
                label: "Same artist penalty",
                max: "1.0",
                oninput: move |e: Event<FormData>| {
                    controller.write().settings.radio.artist_penalty = e.parsed::<f32>().unwrap();
                },
                value: "{controller.read().settings.radio.artist_penalty}",
            }

            SettingsInput {
                max: "1.0",
                label: "Same album penalty",
                oninput: move |e: Event<FormData>| {
                    controller.write().settings.radio.album_penalty = e.parsed::<f32>().unwrap();
                },
                value: "{controller.read().settings.radio.album_penalty}",
            }

            hr {}

            SettingsInput {
                max: "2.0",
                label: "MFCC weight",
                oninput: move |e: Event<FormData>| {
                    controller.write().settings.radio.mfcc_weight = e.parsed::<f32>().unwrap();
                },
                value: "{controller.read().settings.radio.mfcc_weight}",
            }

            SettingsInput {
                max: "2.0",
                label: "Chroma weight",
                oninput: move |e: Event<FormData>| {
                    controller.write().settings.radio.chroma_weight = e.parsed::<f32>().unwrap();
                },
                value: "{controller.read().settings.radio.chroma_weight}",
            }

            SettingsInput {
                max: "2.0",
                label: "Spectral weight",
                oninput: move |e: Event<FormData>| {
                    controller.write().settings.radio.spectral_weight = e.parsed::<f32>().unwrap();
                },
                value: "{controller.read().settings.radio.spectral_weight}",
            }

            SettingsInput {
                max: "2.0",
                label: "Energy weight",
                oninput: move |e: Event<FormData>| {
                    controller.write().settings.radio.energy_weight = e.parsed::<f32>().unwrap();
                },
                value: "{controller.read().settings.radio.energy_weight}",
            }

            SettingsInput {
                max: "2.0",
                label: "BPM weight",
                oninput: move |e: Event<FormData>| {
                    controller.write().settings.radio.bpm_weight = e.parsed::<f32>().unwrap();
                },
                value: "{controller.read().settings.radio.bpm_weight}",
            }

            SettingsInput {
                max: "2.0",
                label: "ZCR weight",
                oninput: move |e: Event<FormData>| {
                    controller.write().settings.radio.zcr_weight = e.parsed::<f32>().unwrap();
                },
                value: "{controller.read().settings.radio.zcr_weight}",
            }
        }
    }
}

#[component]
fn SettingsInput(
    max: String,
    label: String,
    value: String,
    oninput: Callback<Event<FormData>>,
) -> Element {
    rsx! {
        div { class: "settingbox",
            span { "{label}" }
            div { class: "settingsinput",
                input {
                    style: "--dist: calc({value} / {max} * 100.0%);",
                    r#type: "range",
                    max,
                    step: "0.01",
                    value: value.clone(),
                    oninput,
                }

                input { class: "smalltextinput", r#type: "text", value }
            }
        }
    }
}

#[component]
fn LibrarySettings(controller: SyncSignal<MusicController>) -> Element {
    rsx! {
        div { class: "settingsmenu",
            h2 { class: "settingsbar", "Library" }

            div { class: "settingbox",
                span { "Music Directory" }
                input {
                    r#type: "text",
                    value: "{controller.read().settings.directory}",
                    onchange: move |e| controller.write().set_directory(e.value()),
                }
            }
        }
    }
}
