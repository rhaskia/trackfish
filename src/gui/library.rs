use dioxus::{prelude::*, stores::SyncStore};
use crate::app::{MusicController, autotagging::{Recording, get_lastfm_genres, get_possible_track_recordings}, controller::{MusicControllerStoreExt, MusicControllerStoreImplExt}};
use super::DELETING_TRACK;
use super::icons::*;
use super::explorer::ExplorerSwitch;
use super::TagEditor;
use crate::app::track::Track;
use std::time::{Instant, Duration};
use crate::database::{set_tagged, is_tagged, init_db};
use crate::gui::DB;

pub enum LibraryMenu {
    Duplicates,
    Autotagging,
    BulkEditor,
}
    
#[component]
pub fn LibraryManagement(controller: SyncStore<MusicController>) -> Element {
    let mut menu = use_signal(|| LibraryMenu::Duplicates);
    rsx!{
        ExplorerSwitch { controller }

        div { class: "librarymanagementview view", id: "librarymanagement",
            div { class: "header",
                button {
                    class: "basicbutton",
                    onclick: move |_| menu.set(LibraryMenu::Duplicates),
                    "Duplicates"
                }
                button {
                    class: "basicbutton",
                    onclick: move |_| menu.set(LibraryMenu::Autotagging),
                    "Autotagging"
                }
                button {
                    class: "basicbutton",
                    onclick: move |_| menu.set(LibraryMenu::BulkEditor),
                    "Bulk Editor"
                }
            }

            match *menu.read() {
                LibraryMenu::Duplicates => rsx! {
                    DuplicateMenu { controller }
                },
                LibraryMenu::Autotagging => rsx! {
                    TaggingMenu { controller }
                },
                LibraryMenu::BulkEditor => rsx! {
                    BulkEditor { controller } 
                }
            }
        }
    }
}

#[component]
pub fn BulkEditor(controller: SyncStore<MusicController>) -> Element {
    let mut changes = use_signal(Vec::new);
    //let change = use_signal(String::new);

    // let find_containing = move |_| {
    //     for i in 0..controller.all_tracks().read().len() {
    //         if controller.all_tracks().get(i).unwrap().read().artists.iter().any(|a| a.ends_with("- Topic")) {
    //             let artists = controller.all_tracks().get(i).unwrap().read().artists.clone();
    //             let changed_artists = artists.iter().map(|a| a.replace(" - Topic", "")).collect::<Vec<String>>();
    //             changes.push((i, artists, changed_artists));
    //         }
    //     }
    // };

    // let find_containing = move |_| {
    //     for i in 0..controller.all_tracks().read().len() {
    //         if controller.all_tracks().get(i).unwrap().read().artists.iter().any(|a| a.contains(",") || a.contains("&")) {
    //             let artists = controller.all_tracks().get(i).unwrap().read().artists.clone();
    //             let changed_artists = artists.iter().map(|a| a.split([',', '&']).map(|s| s.trim().to_string()).collect::<Vec<String>>()).flatten().collect::<Vec<String>>();
    //             let mut no_dups = Vec::new();
    //             for artist in &changed_artists {
    //                 if !no_dups.contains(artist) {
    //                     no_dups.push(artist.clone())
    //                 }
    //             }
    //             changes.push((i, artists, no_dups));
    //         }
    //     }
    // };

    // let find_containing = move |_| {
    //     for i in 0..controller.all_tracks().read().len() {
    //         if controller.all_tracks().get(i).unwrap().read().title.contains(" - ") {
    //             let original = controller.all_tracks().get(i).unwrap()();
    //             let mut changed = original.clone();
    //             let mut s = original.title.split(" - ");
    //             let artist = s.next().unwrap();
    //             let title = s.next().unwrap();
    //             changed.title = title.to_string();
    //             changed.artists = vec![artist.to_string()];
    //             changes.push((i, original, changed));
    //         }
    //     }
    // };

    let find_containing = move |_| {
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
                onclick: find_containing,
                margin: "0 10px",
                "Load edits"
            },
            button {
                class: "basicbutton",
                onclick: save_changes,
                margin: "0 10px",
                "Save changes"
            }
        }
        div {
            class: "bulkeditor",
            div {
                "{changes.read().len()} possible edits",
            }
            div {
                class: "bulkeditorlist",
                for (index, (i, original, changed)) in changes.read().iter().enumerate() {
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

#[component]
pub fn DuplicateMenu(controller: SyncStore<MusicController>) -> Element {
    let mut duplicates: Signal<Vec<Vec<usize>>> = use_signal(|| Vec::new());

    rsx!{
        button {
            class: "basicbutton",
            onclick: move |_| duplicates.set(controller.find_duplicates()),
            margin: "0 10px",
            "Load duplicates"
        }
        div { class: "duplicates",
            for i in 0..duplicates.read().len() {
                div { class: "duplicategroup",
                    img {
                        onclick: move |_| {
                            duplicates.remove(i);
                        },
                        margin: "15px 0px auto 5px",
                        margin_bottom: "auto",
                        class: "trackbutton",
                        loading: "lazy",
                        src: CLOSE_ICON,
                    }
                    div { class: "duplicatesongs",
                        for j in 0..duplicates.read()[i].len().min(20) {
                            div { class: "duplicatesong",
                                img {
                                    class: "trackitemicon",
                                    loading: "onvisible",
                                    src: "/trackimage/{duplicates.read()[i][j]}?origin=library",
                                }
                                "{controller.read().all_tracks[duplicates.read()[i][j]].title} - "
                                "{controller.read().all_tracks[duplicates.read()[i][j]].artists.join(\", \")}"
                                div { flex_grow: 1 }
                                img {
                                    onclick: move |_| DELETING_TRACK.set(Some(duplicates.read()[i][j])),
                                    class: "trackbutton",
                                    loading: "lazy",
                                    src: DELETE_ICON,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

enum TaggingType {
    All,
    NoAlbum,
    BadGenres,
    BadGenresNoAlbum
}

fn bad_genre(genre: &str) -> bool {
    genre == "Trance" || genre == "Electronic" || genre == "Jazz" ||
     genre == "Pop" || genre == "Ambient" || genre == "Rnb"
}

fn bad_genres(track: &Track) -> bool {
    track.genres.iter().filter(|g| bad_genre(g)).count() >= 2
}

#[component]
pub fn TaggingMenu(controller: SyncStore<MusicController>) -> Element {
    // tagging all untagged tracks or just those missing metadata
    let tagging_all = use_signal(|| TaggingType::All);
    let mut tags: Signal<Vec<Recording>> = use_signal(|| Vec::new());
    let mut index = use_signal(|| 0);
    let mut tag_index = use_signal(|| 0);
    let title = use_memo(move || controller.read().all_tracks[index()].title.clone());
    let artist = use_memo(move || controller.read().all_tracks[index()].artists.join(", "));
    let mut lastfm_genres = use_signal(Vec::new);

    let mut cached_index = use_signal(|| 0);
    let mut cache = use_signal(Vec::new);

    let mut tag = use_signal(Track::default);
    let mut started = use_signal(|| false);

    let mut with_bad_genres = use_signal(Vec::new);
    let mut no_album = use_signal(Vec::new);
    let mut with_no_albums = use_signal(|| 0);
    let mut no_genres = use_signal(Vec::new);
    let mut reset = use_signal(|| false);

    use_future(move || async move {
        println!("getting tag");

        for i in 0..controller.all_tracks().read().len() {
            let track = controller.all_tracks().get(i).unwrap();
            let album = &track.read().album;

            if album.is_empty() || album.to_ascii_lowercase() == "music" {
                no_album.push(i);
            }

            if track.read().genres.len() == 0 || track.read().genres[0].is_empty() {
                no_genres.push(i);
            }

            if bad_genres(&*track.read()) {
                with_bad_genres.push(i);
            }
        }

        with_no_albums.set(no_album.len());

        loop {
            if cache.read().len() > 10 {
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }

            if is_tagged(&*DB.read(), &controller.all_tracks().get(cached_index()).unwrap().read().file).unwrap() {
                if no_album.read().len() > 0 {
                    cached_index.set(no_album.write().pop().unwrap());
                } else {
                    if !reset() {
                        reset.set(true);
                        cached_index.set(0);
                    } else {
                        *cached_index.write() += 1;
                    }
                }
                continue;
            }

            info!("{:?} isnt tagged", controller.all_tracks().get(cached_index()).unwrap().read().file);

            let last_requested = Instant::now();
            let recordings = get_possible_track_recordings(controller.all_tracks().get(cached_index()).unwrap()()).await;

            match recordings {
                Ok(r) => {
                    cache.write().push((cached_index(), r));
                    if no_album.read().len() > 0 {
                        info!("no album poppped");
                        cached_index.set(no_album.write().pop().unwrap());
                    } else {
                        if !reset() {
                            reset.set(true);
                            cached_index.set(0);
                        } else {
                            *cached_index.write() += 1;
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
                Err(err) => {
                    log::info!("{err:?}");
                }
            }

            if !started() && cache.read().len() > 0 {
                let next = cache.pop().unwrap();
                tags.set(next.1);
                index.set(next.0);
                started.set(true);
                tag.set(controller.all_tracks().get(index()).unwrap()());
            }

            if last_requested.elapsed() < Duration::from_millis(1100) {
                tokio::time::sleep(Duration::from_millis(1100) - last_requested.elapsed()).await;
            }
        }
    });

    let next = move |_: Event<MouseData>| async move {
        info!("next started");
        set_tagged(&*DB.read(), &tag.read().file).unwrap();
        tag_index.set(0);
        let next = cache.write().pop().unwrap();
        index.set(next.0);
        tags.set(next.1);
        let new_tag = controller.read().all_tracks[index()].clone();
        tag.set(new_tag);
        info!("next done");
    };

    rsx!{
        div { class: "taggingview",
            "{with_no_albums()} tracks without albums"
            br {}
            "{no_genres.read().len()} tracks without genres"
            br {}
            "{with_bad_genres.read().len()} tracks with bad genres"

            div { class: "tagsidebyside",
                div { class: "oldtags tags",
                    TagEditor { controller, tag, index: index() }
                }

                div { class: "newtags tags",
                    if tags.read().len() > 0 {
                        "{tag_index + 1}/{tags.read().len()}"
                        div { class: "tag newtag",
                            div { class: "editorline",
                                label { r#for: "title", "Title" }
                                input {
                                    disabled: true,
                                    name: "title",
                                    id: "title",
                                    r#type: "text",
                                    value: "{tags.read()[tag_index()].title()}",
                                }
                            }
                            div { class: "editorline",
                                label { r#for: "album", "Album" }
                                input {
                                    disabled: true,
                                    name: "album",
                                    id: "album",
                                    r#type: "text",
                                    value: "{tags.read()[tag_index()].album()}",
                                }
                            }
                            div { class: "editorline",
                                label { r#for: "artists", "Artists" }
                                input {
                                    disabled: true,
                                    name: "artists",
                                    id: "artists",
                                    r#type: "text",
                                    value: "{tags.read()[tag_index()].artists():?}",
                                }
                            }
                            div { class: "editorline",
                                label { r#for: "genres", "Genres" }
                                input {
                                    disabled: true,
                                    name: "genres",
                                    id: "genres",
                                    r#type: "text",
                                    value: "{tags.read()[tag_index()].genres():?}",
                                }
                            }
                        }

                        button {
                            disabled: tag_index() == tags.read().len() - 1,
                            onclick: move |_| *tag_index.write() += 1,
                            "Next"
                        }

                        button {
                            disabled: tag_index() < 1,
                            onclick: move |_| *tag_index.write() -= 1,
                            "Previous"
                        }

                        button {
                            onclick: move |_| {
                                tag.write().title = tags.read()[tag_index()].title().to_string();
                                tag.write().album = tags.read()[tag_index()].album().to_string();
                                tag.write().artists = tags.read()[tag_index()].artists();
                                tag.write().genres = tags.read()[tag_index()].genres();
                            },
                            "Use Information"
                        }

                        button {
                            onclick: move |_| {
                                tag.write().genres = tags.read()[tag_index()].genres();
                            },
                            "Use Genres"
                        }

                        button {
                            onclick: move |_| async move {
                                let api_key = controller.settings().read().tagging.lastfm_key.clone();
                                let binding = tags.read();
                                let track = binding[tag_index()].title();
                                let artist = binding[tag_index()].artists()[0].clone();
                                lastfm_genres.set(get_lastfm_genres(track, &artist, &api_key).await.unwrap())
                            },
                            "Request lastfm genres"
                        }

                        a { 
                            color: "var(--accent)",
                            href: "https://last.fm/music/{tag.read().artists.get(0).cloned().unwrap_or_default()}/_/{tag.read().title}",
                            "LastFM Page" 
                        }

                        button { "Request cover art" }
                    }

                    "{lastfm_genres:?}"
                }
            }

            div { class: "tagchoices",
                button { disabled: cache.read().is_empty(), onclick: next, "Ignore" }
                button {
                    disabled: cache.read().is_empty(),
                    class: "accentbutton",
                    onclick: move |e| async move {
                        let db = &*DB.read();
                        controller.update_tag(db, index(), tag());
                        info!("updated tag");
                        next(e).await;
                        info!("finished tag");
                        *with_no_albums.write() -= 1;
                    },
                    "Confirm"
                }
            }

            div { class: "customtagsearch",
                a { href: "https://musicbrainz.org/taglookup/index?tag-lookup.artist={artist}&tag-lookup.track={title}",
                    "Use online musicbrainz search"
                }
                label { r#for: "mbid", "Enter custom mbid:" }
                input { r#type: "text", name: "mbid" }
                button { "Lookup" }
            }
        }
    }
}