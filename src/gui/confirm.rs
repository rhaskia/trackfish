use dioxus::prelude::*;

#[component]
pub fn Confirmation(
    label: String,
    confirm: Callback<Event<MouseData>>,
    cancel: Callback<Event<MouseData>>,
    visible: bool,
) -> Element {
    rsx! {
        div { class: "optionsbg",
            display: if !visible { "none" },
            onclick: cancel,
            div { class: "optionbox", style: "--width: 250px; --height: 50px",
                h3 { "{label}" }
                div { display: "flex",
                    button { onclick: confirm, "Confirm" }
                    button { "Cancel" }
                }
            }
        }
    }
}
