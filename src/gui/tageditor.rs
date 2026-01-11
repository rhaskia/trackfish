use dioxus::prelude::*;
use crate::app::MusicController;
use crate::gui::EDITING_TAG;
use crate::app::track::Track;

#[component]
pub fn TagEditor(controller: SyncSignal<MusicController>) -> Element {
    let mut tag = use_signal(Track::default);

    use_effect(move || {
        if let Some(track) = EDITING_TAG() {
            info!("new track to edit");
            tag.set(track.1);
        }
    });

    rsx!{
        if EDITING_TAG().is_some() {
            div { class: "editorbg", // onclick: move |_| EDITING_TAG.set(None),
                div {
                    onclick: |e| e.stop_propagation(),
                    class: "editorbox",
                    style: "--width: 300px; --height: 300px",

                    img { src: "/trackimage/{EDITING_TAG().unwrap().0}" }

                    div { class: "editorline",
                        label { r#for: "title", "Title" }
                        input {
                            name: "title",
                            id: "title",
                            r#type: "text",
                            value: "{tag.read().title}",
                            oninput: move |e| {
                                tag.write().title = e.value();
                            },
                        }
                    }

                    div { class: "editorline",
                        label { r#for: "album", "Album" }
                        input {
                            name: "album",
                            id: "album",
                            r#type: "text",
                            value: "{tag.read().album}",
                            oninput: move |e| tag.write().album = e.value(),
                        }
                    }

                    div { class: "editormultiple",
                        div { class: "editormultipleline",
                            label { "Artists" }
                            button {
                                onclick: move |_| {
                                    let split = tag
                                        .read()
                                        .artists[0]
                                        .split(&[',', '&'][..])
                                        .map(|s| s.trim().to_string())
                                        .collect();
                                    tag.write().artists = split;
                                },
                                "Split by comma"
                            }
                        }
                        for i in 0..tag.read().artists.len() {
                            div { class: "editormultipleline",
                                input {
                                    flex: "1 1 0",
                                    id: "artist",
                                    r#type: "text",
                                    value: "{tag.read().artists[i]}",
                                    oninput: move |e| tag.write().artists[i] = e.value(),
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
                        label { "Genres" }
                        for i in 0..tag.read().genres.len() {
                            div { class: "editormultipleline",
                                input {
                                    flex: "1 1 0",
                                    id: "genre",
                                    r#type: "text",
                                    value: "{tag.read().genres[i]}",
                                    onchange: move |e| tag.write().genres[i] = e.value(),
                                }
                                button {
                                    onclick: move |_| {
                                        tag.write().genres.remove(i);
                                    },
                                    "Remove"
                                }
                            }
                        }

                        button { onclick: move |_| tag.write().genres.push(String::new()),
                            "+ Genre"
                        }
                    }

                    div { class: "editoroptions",
                        button { onclick: move |_| EDITING_TAG.set(None), "Cancel" }

                        button {
                            background: "var(--accent)",
                            onclick: move |_| {
                                controller.write().update_tag(EDITING_TAG().unwrap().0, tag());
                                EDITING_TAG.set(None);
                            },
                            "Confirm"
                        }
                    }
                }
            }
        }
    }
}
