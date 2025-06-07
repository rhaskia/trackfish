use dioxus::prelude::*;

#[component]
pub fn Confirmation(
    label: String,
    confirm: Callback<Event<MouseData>>,
    cancel: Callback<Event<MouseData>>,
) -> Element {
    rsx! {
        div { class: "optionsbg", onclick: cancel,
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
