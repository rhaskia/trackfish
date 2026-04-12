use dioxus::{prelude::*, stores::SyncStore};
use crate::app::MusicController;
use crate::app::controller::MusicControllerStoreExt;
use crate::gui::icons::*;

#[derive(Clone, Debug)]
enum BulkEditorTool {
    RemoveTopic,
    SplitArtists,
    SplitTitleArtist,
    RemoveUnneededBrackets,
}

/// Splits up artists into separate strings if connected
/// Only works if the conjuctions are commas, ampersands, or feat./ft.
pub fn split_artists_complete(artists: Vec<String>) -> Vec<String> {
    let changed_artists = artists.iter()
        .map(|a| a.split([',', '&']).map(|s| s.trim().to_string()).collect::<Vec<String>>())
        .flatten().collect::<Vec<String>>();

    let changed_artists = changed_artists.iter()
        .map(|a| a.split("feat.").map(|s| s.trim().to_string()).collect::<Vec<String>>())
        .flatten().collect::<Vec<String>>();

    let changed_artists = changed_artists.iter()
        .map(|a| a.split("ft.").map(|s| s.trim().to_string()).collect::<Vec<String>>())
        .flatten().collect::<Vec<String>>();

    let mut no_dups = Vec::new();
    for artist in &changed_artists {
        if !no_dups.contains(artist) {
            no_dups.push(artist.clone())
        }
    }

    return no_dups;
}

#[component]
pub fn BulkEditor(controller: SyncStore<MusicController>) -> Element {
    let mut changes = use_signal(Vec::new);
    let mut change = use_signal(|| BulkEditorTool::SplitArtists);

    // Removes - topic from artist names
    let mut remove_topic = move || {
        for i in 0..controller.all_tracks().read().len() {
            if controller.all_tracks().get(i).unwrap().read().artists.iter().any(|a| a.ends_with("- Topic")) {
                let original = controller.all_tracks().get(i).unwrap()();
                let mut changed = original.clone();

                let artists = original.artists.clone();
                let changed_artists = artists.iter().map(|a| a.replace(" - Topic", "")).collect::<Vec<String>>();
                changed.artists = changed_artists;
                changes.push((i, original, changed));
            }
        }
    };

    // Splits up unseparated artist strings in metadata
    let mut split_artists = move || {
        for i in 0..controller.all_tracks().read().len() {
            let track = controller.all_tracks().get(i).unwrap();

            if track.read().artists.iter().any(|a| a.to_lowercase().contains("tyler, the creator")) {
                continue;
            }

            if track.read().artists.iter().any(|a| a.contains(",") || a.contains(" & ") || a.contains("feat.") || a.contains("ft.")) {
                let original = track();
                let mut changed = original.clone();

                let artists = original.artists.clone();
                changed.artists = split_artists_complete(artists);

                changes.push((i, original, changed));
            }
        }
    };

    // If the title of a track is Artist - Title, this helps to properly set the metadata
    let mut split_title_artist = move || {
        for i in 0..controller.all_tracks().read().len() {
            if controller.all_tracks().get(i).unwrap().read().title.contains(" - ") {
                let original = controller.all_tracks().get(i).unwrap()();
                let mut changed = original.clone();
                
                let mut s = original.title.split(" - ");
                let artist = s.next().unwrap();
                let title = s.next().unwrap();
                changed.title = title.to_string();
                changed.artists = vec![artist.to_string()];

                changes.push((i, original, changed));
            }
        }
    };

    // Removes brackets containing official (audio/visualizer/video) or audio in brackets
    // Possibly could be extended
    let mut remove_unneeded_brackets = move || {
        for i in 0..controller.all_tracks().read().len() {
            let title = controller.all_tracks().get(i).unwrap().read().title.to_ascii_lowercase();
            if title.contains("official") || title.contains("audio") {
                let original = controller.all_tracks().get(i).unwrap()();
                let mut changed = original.clone();
                let mut s = original.title.split(['[', '(']);
                let title = s.next().unwrap().trim();
                changed.title = title.to_string();
                changes.push((i, original, changed));
            }
        }
    };

    // Saves bulk edit changes to database and files
    let save_changes = move |_| {
        let database = crate::database::init_db().unwrap();
        for change in &*changes.read() {
            if let Some(ref mut track) = controller.all_tracks().get(change.0) {
                track.set(change.2.clone());
                track.write().save_to_disk(&database).unwrap();
                log::info!("{:?}", track);
                log::info!("{:?} => {:?}", change.1, change.2);
            }
        }
    };

    rsx!{
        div {
            button {
                class: "basicbutton",
                onclick: move |_| {
                    changes.set(Vec::new());
                    use BulkEditorTool::*;
                    match change() {
                        SplitArtists => split_artists(),
                        SplitTitleArtist => split_title_artist(),
                        RemoveTopic => remove_topic(),
                        RemoveUnneededBrackets => remove_unneeded_brackets(),
                    }
                },
                margin: "0 10px",
                "Load edits"
            },
            button {
                class: "basicbutton",
                onclick: save_changes,
                margin: "0 10px",
                "Save changes"
            }
            select {
                onchange: move |e| {
                    match e.value().as_str() {
                        "Split Artists" => change.set(BulkEditorTool::SplitArtists),
                        "Split Title Artist" => change.set(BulkEditorTool::SplitTitleArtist),
                        "Remove Topic" => change.set(BulkEditorTool::RemoveTopic),
                        "Remove Unneeded Brackets" => change.set(BulkEditorTool::RemoveUnneededBrackets),
                        data => info!("{data}")
                    }
                },
                option { "Split Artists" }
                option { "Split Title Artist" }
                option { "Remove Topic" }
                option { "Remove Unneeded Brackets" }
            }
            "{change:?}"
        }
        div {
            class: "bulkeditor",
            div {
                "{changes.read().len()} possible edits",
            }
            div {
                class: "bulkeditorlist",
                for (index, (_, original, changed)) in changes.read().iter().enumerate() {
                    div {
                        class: "bulkeditoritem",
                        img {
                            onclick: move |_| {
                                changes.remove(index);
                            },
                            margin: "5px",
                            margin_bottom: "auto",
                            class: "trackbutton",
                            loading: "lazy",
                            src: CLOSE_ICON,
                        },
                        "{original.title:?} => {changed.title:?}",
                        br {}
                        "{original.artists:?} => {changed.artists:?}",
                    }
                }
            }
        }
    }
}