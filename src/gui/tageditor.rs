use dioxus::prelude::*;
use crate::app::MusicController;
use crate::gui::EDITING_TAG;
use crate::app::track::Track;

#[component]
pub fn TagEditor(controller: SyncSignal<MusicController>) -> Element {
    let mut tag = use_signal(Track::default);

    use_effect(move || {
        if let Some(track) = EDITING_TAG() {
            tag.set(controller.read().all_tracks[track].clone());
        }
    });

    rsx!{
        if EDITING_TAG().is_some() {
            div { class: "editorbg", onclick: move |_| EDITING_TAG.set(None),
                div {
                    onclick: |e| e.stop_propagation(),
                    class: "editorbox",
                    style: "--width: 300px; --height: 300px",

                    img { src: "/trackimage/{EDITING_TAG().unwrap()}" }

                    div { class: "editorline",
                        label { r#for: "title", "Title" }
                        input {
                            name: "title",
                            id: "title",
                            r#type: "text",
                            value: "{tag.read().title}",
                            onchange: move |e| tag.write().title = e.value(),
                        }
                    }

                    div { class: "editorline",
                        label { r#for: "album", "Album" }
                        input {
                            name: "album",
                            id: "album",
                            r#type: "text",
                            value: "{tag.read().album}",
                            onchange: move |e| tag.write().album = e.value(),
                        }
                    }

                    div { class: "editormultiple",
                        label { r#for: "artist", "Artists" }
                        for i in 0..tag.read().artists.len() {
                            div { class: "editormultipleline",
                                input {
                                    flex: "1 1 0",
                                    name: "album",
                                    id: "artist",
                                    r#type: "text",
                                    value: "{tag.read().artists[i]}",
                                    onchange: move |e| tag.write().artists[i] = e.value(),
                                }
                                button {
                                    onclick: move |_| {
                                        tag.write().artists.remove(i);
                                    },
                                    "Remove"
                                }
                            }
                        }

                        button { onclick: move |_| tag.write().artists.push(String::new()),
                            "+ Artist"
                        }
                    }

                    div { class: "editormultiple",
                        label { r#for: "artist", "Artists" }
                        for i in 0..tag.read().artists.len() {
                            div { class: "editormultipleline",
                                input {
                                    flex: "1 1 0",
                                    name: "album",
                                    id: "artist",
                                    r#type: "text",
                                    value: "{tag.read().artists[i]}",
                                    onchange: move |e| tag.write().artists[i] = e.value(),
                                }
                                button {
                                    onclick: move |_| {
                                        tag.write().artists.remove(i);
                                    },
                                    "Remove"
                                }
                            }
                        }

                        button { onclick: move |_| tag.write().artists.push(String::new()),
                            "+ Artist"
                        }
                    }

                    div { class: "editoroptions",
                        button { onclick: move |_| EDITING_TAG.set(None), "Cancel" }

                        button {
                            background: "var(--accent)",
                            onclick: move |_| controller.write().update_tag(EDITING_TAG().unwrap(), tag()),
                            "Confirm"
                        }
                    }
                }
            }
        }
    }
}
