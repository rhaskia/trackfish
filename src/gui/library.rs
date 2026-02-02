use dioxus::{prelude::*, stores::SyncStore};
use crate::app::{MusicController, autotagging::{get_possible_track_recordings, get_lastfm_genres, Recording}};
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
    Autotagging
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
            }

            match *menu.read() {
                LibraryMenu::Duplicates => rsx! {
                    DuplicateMenu { controller }
                },
                LibraryMenu::Autotagging => rsx! {
                    TaggingMenu { controller }
                },
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
            onclick: move |_| duplicates.set(controller.read().find_duplicates()),
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

#[component]
pub fn TaggingMenu(controller: SyncStore<MusicController>) -> Element {
    // tagging all untagged tracks or just those missing metadata
    let tagging_all = use_signal(|| true);
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

    use_future(move || async move {
        println!("getting tag");

        loop {
            if is_tagged(&*DB.read(), &controller.read().all_tracks[cached_index()].file).unwrap() {
                *cached_index.write() += 1;
                continue;
            }
            info!("{:?} isnt tagged", controller.read().all_tracks[cached_index()].file);

            let last_requested = Instant::now();
            let recordings = get_possible_track_recordings(controller.read().all_tracks[cached_index()].clone()).await;

            match recordings {
                Ok(r) => {
                    cache.write().push((cached_index(), r));
                    *cached_index.write() += 1;
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
                tag.set(controller.read().all_tracks[index()].clone());
            }

            if last_requested.elapsed() < Duration::from_millis(1100) {
                tokio::time::sleep(Duration::from_millis(1100) - last_requested.elapsed()).await;
            }
        }
    });

    let next = move |_: Event<MouseData>| async move {
        set_tagged(&*DB.read(), &tag.read().file).unwrap();
        tag_index.set(0);
        let next = cache.write().pop().unwrap();
        index.set(next.0);
        tags.set(next.1);
        let new_tag = controller.read().all_tracks[index()].clone();
        tag.set(new_tag);
    };

    rsx!{
        div { class: "taggingview",

            div { class: "tagsidebyside",
                div { class: "oldtags tags",
                    TagEditor { controller, tag, index: index() }
                }

                div { class: "newtags tags",
                    if tags.read().len() > 0 {
                        "{tag_index + 1}/{tags.read().len()}"
                        div { class: "tag newtag",
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
                            span { "{tags.read()[tag_index()].title()}" }
                            span { "{tags.read()[tag_index()].artists():?}" }
                            span { "{tags.read()[tag_index()].genres():?}" }
                        }

                        button {
                            disabled: tag_index() == tags.read().len() - 1,
                            onclick: move |_| *tag_index.write() += 1,
                            "Next"
                        }

                        button {
                            disabled: tag_index() == 0,
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
                                let api_key = &controller.read().settings.tagging.lastfm_key;
                                let binding = tags.read();
                                let track = binding[tag_index()].title();
                                let artist = binding[tag_index()].artists()[0].clone();
                                lastfm_genres.set(get_lastfm_genres(track, &artist, api_key).await.unwrap())
                            },
                            "Request lastfm genres"
                        }

                        button { "Request cover art" }
                    }
                }
            }

            div { class: "tagchoices",
                button { disabled: cache.read().is_empty(), onclick: next, "Ignore" }
                button {
                    disabled: cache.read().is_empty(),
                    class: "accentbutton",
                    onclick: move |e| async move {
                        let db = &*DB.read();
                        let mut binding = controller.write();
                        binding.update_tag(db, index(), tag());
                        next(e).await;
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