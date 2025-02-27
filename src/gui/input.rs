use dioxus::prelude::*;

pub fn key_to_action(event: Event<KeyboardData>) -> Option<Action> {
    let key = event.data().key();
    match key {
        Key::Character(c) => match c.as_str() {
            "l" => Some(Action::Skip),
            "h" => Some(Action::SkipPrevious),
            " " => Some(Action::PauseToggle),
            _ => None,
        },
        _ => None
    }
}

pub enum Action {
    Skip,
    SkipPrevious,
    Play,
    Pause,
    PauseToggle,
    Shuffle,
    Stop
}
