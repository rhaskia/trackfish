use std::time::{Duration, Instant};

use dioxus::prelude::*;
use dioxus::stores::SyncStore;
use crate::app::controller::{MusicControllerStoreExt, MusicControllerStoreImplExt};
use crate::database::{set_tagged, is_tagged};
use crate::app::{MusicController, Track, autotagging::{Recording, get_possible_track_recordings}};
use crate::gui::{DB, TagEditor};

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
    let mut tags: Signal<Vec<Recording>> = use_signal(|| Vec::new());
    let mut index = use_signal(|| 0);
    let mut tag_index = use_signal(|| 0);
    let title = use_memo(move || controller.read().all_tracks[index()].title.clone());
    let artist = use_memo(move || controller.read().all_tracks[index()].artists.join(", "));

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
            "{with_no_albums()} tracks without albums, ",
            "{no_genres.read().len()} tracks without genres, ",
            "{with_bad_genres.read().len()} tracks with bad genres",

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
                    }
                    
                    a { 
                        color: "var(--accent)",
                        href: "https://last.fm/music/{tag.read().artists.get(0).cloned().unwrap_or_default()}/_/{tag.read().title}",
                        "LastFM Page" 
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