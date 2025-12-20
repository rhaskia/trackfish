use std::io;
use std::path::PathBuf;

use dioxus::desktop::use_asset_handler;
use dioxus::prelude::*;
use dioxus::document::eval;
use http::Response;

fn main() {
    launch(App);
}

#[component]
pub fn App() -> Element {
    let tracks = use_signal(|| get_song_files("E:/music").unwrap());
    let mut window_size = use_signal(|| 0);
    const ROW_HEIGHT: usize = 62;
    const BUFFER_ROWS: usize = 5;

    let mut start_index = use_signal(|| 0);
    let rows_in_view = use_memo(move || window_size() / ROW_HEIGHT + BUFFER_ROWS);
    let end_index = use_memo(move || (start_index() + rows_in_view()).min(tracks.read().len()));

    use_asset_handler("trackimage", move |request, responder| {
        let r = Response::builder().status(404).body(&[]).unwrap();

        let id: usize = if let Ok(id) = request.uri().path().replace("/trackimage/", "").parse() {
            id
        } else {
            responder.respond(r);
            return;
        };

        let file = if let Some(file) = get_track_image(&tracks.read()[id].clone()) {
            file
        } else {
            responder.respond(r);
            return;
        };

        responder.respond(Response::builder().body(file).unwrap());
    });

    use_future(move || async move {
        let mut js = eval(
            r#"
            new ResizeObserver(() => {
                let container = document.getElementById("alltrackslist");
                dioxus.send(container.offsetHeight);
            }).observe(document.getElementById("alltrackslist"));
        "#,
        );

        loop {
            let height = js.recv::<usize>().await;
            if let Ok(height) = height {
                window_size.set(height);
                info!("window height {height}");
            }
        }
    });

    use_effect(move || {
        let mut js = eval(
            r#"
            let container = document.getElementById('main');
            container.addEventListener('scroll', function(event) {
                dioxus.send(container.scrollTop);
            });
        "#,
        );

        spawn(async move {
            loop {
                let scroll_top = js.recv::<usize>().await;
                println!("{scroll_top:?}");
                if let Ok(scroll_top) = scroll_top {
                    let new_index = (scroll_top as f32 / ROW_HEIGHT as f32).floor() as usize;
                    if new_index != start_index() {
                        start_index.set(new_index);
                    }
                }
            }
        });
    });

    rsx! {
        div { min_height: "{1000 * ROW_HEIGHT}px" }

        for i in start_index()..end_index() {
            div {
                class: "trackitem",
                id: "alltracks-trackitem-{i}",
                style: "top: {i - start_index()}px; position: absolute;",
                height: "62px",
                position: "absolute",
                img {
                    class: "trackitemicon",
                    loading: "lazy",
                    height: "63px",
                    src: "/trackimage/{i}",
                }
            }
        }
    }
}

fn get_song_files(directory: &str) -> Result<Vec<String>, io::Error> {
    // Can't seem to load paths with tildes in them
    let expanded = if let Some(home) = dirs::home_dir() {
        directory.replace("~", &home.display().to_string())
    } else {
        directory.to_string()
    };

    let files = recursive_read_dir(&expanded)?;

    Ok(files)
}

/// Recursively reads a directory
fn recursive_read_dir(dir: &str) -> Result<Vec<String>, io::Error> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let filename = path.to_str().unwrap().to_string();

        match path.is_file() {
            true => {
                if path_is_audio(path) {
                    files.push(filename.into());
                }
            }
            false => {
                let mut dir_files = recursive_read_dir(&filename)?;
                files.append(&mut dir_files);
            }
        }
    }

    Ok(files)
}

/// Is a file an audio file?
fn path_is_audio(path: PathBuf) -> bool {
    match path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
    {
        "mp3" | "opus" | "wav" | "flac" | "ogg" | "aiff" => true,
        _ => false,
    }
}

/// Returns the track image information from metadata as bytes
pub fn get_track_image(file: &str) -> Option<Vec<u8>> {
    let filetype = file.split(".").last()?;

    match filetype {
        "flac" => {
            // let tag = metaflac::Tag::read_from_path(&file).ok()?;
            //
            // for frame in tag.blocks() {
            //     if let Block::Picture(picture) = frame {
            //         return Some(picture.data.clone());
            //     }
            // }

            None
        }
        "ogg" => {
            // let f = std::fs::File::open(file).ok()?;
            // let tag = lewton::inside_ogg::OggStreamReader::new(f).ok()?;
            //
            // let comments = tag.comment_hdr;
            // let picture = comments.comment_list.iter().find(|(k, v)| k == "METADATA_BLOCK_PICTURE")?;
            //
            // let mut encoded = picture.1.clone();
            // for _ in 0..(encoded.len() % 4) {
            //     encoded.push('=');
            // }
            //
            // let decoded = base64::prelude::BASE64_STANDARD
            //     .decode(encoded.as_bytes()).unwrap();
            // let picture = metaflac::block::Picture::from_bytes(&decoded).ok()?;
            //
            // Some(picture.data)
            None
        }
        _ => {
            let tag = id3::Tag::read_from_path(file).ok()?;
            let picture = tag.pictures().next()?;
            Some(picture.data.clone())
        }
    }
}