//#![feature(trivial_bounds)]

pub mod audio;
pub mod queue;
pub mod track;
pub mod embed;

//use diesel::prelude::*;
use dioxus::prelude::*;
//use dotenvy::dotenv;
use id3::Tag;
use id3::TagLike;
use log::Level;
use log::error;
use queue::QueueManager;
use track::load_tracks;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::DirEntry;
use std::io;
use std::io::Cursor;
use std::time::SystemTime;

use http::{header::*, response::Builder as ResponseBuilder, status::StatusCode};
use std::io::SeekFrom;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::time;
use tokio::time::Duration;
use tokio::runtime::Runtime;
use log::info;
use android_logger::Config;
use http::Response;
use tracing_log::LogTracer;
use log::LevelFilter;

#[cfg(not(target_os = "android"))]
use dioxus::desktop::{use_asset_handler, AssetRequest};
#[cfg(target_os = "android")]
use dioxus::mobile::{use_asset_handler, AssetRequest};

use audio::AudioPlayer;
use track::Track;

fn main() {
    if cfg!(android) {
       android_logger::init_once(
            Config::default().with_max_level(LevelFilter::Trace),
       );
    }

    LogTracer::init().expect("Failed to initialize LogTracer");

    dioxus_logger::init(dioxus_logger::tracing::Level::INFO);

    launch(App);
}

const CURRENT: GlobalSignal<usize> = GlobalSignal::new(|| 19);
// //const DB: GlobalSignal<SqliteConnection> = GlobalSignal::new(|| establish_connection());
// const CURRENT_TRACK: GlobalSignal<Option<Track>> = GlobalSignal::new(|| None);
const DIR: GlobalSignal<&str> = GlobalSignal::new(|| {
    if cfg!(target_os = "android") {
        "/storage/emulated/0/Music/"
    } else {
        "E:/Music/"
    }
});

const TRACKS: GlobalSignal<Vec<Track>> = GlobalSignal::new(|| Vec::new());

#[component]
fn App() -> Element {
    let mut queue = use_signal(|| QueueManager::new(Vec::new()));

    use_future(move || async move { 
        info!("Requested storage permissions: {:?}", crossbow::Permission::StorageWrite.request_async().await);
        queue.set(QueueManager::new(load_tracks(DIR())));
        info!("loaded all tracks into queue manager");
    });

    use_asset_handler("trackimage", move |request, responder| {
        info!("{:?}", request.uri());
        let id = request.uri().path().replace("/trackimage/", "");
        let path = &TRACKS.read()[CURRENT()].file;
        info!("{path}");
        let tag = Tag::read_from_path(path).unwrap();
        let mut file = Cursor::new(tag.pictures().next().unwrap().data.clone());

        tokio::task::spawn(async move {
            match get_stream_response(&mut file, &request).await {
                Ok(response) => responder.respond(response),
                Err(err) => eprintln!("Error: {}", err),
            }
        });
    });

    rsx! {
        style {{ include_str!("../assets/style.css") }}

        div {
            class: "mainview",
            SongView { queue }

            // div {
            //     class: "listensview",
            //     //"Next Up: {queue.read().next_up().title}"
            // }
        }

        MenuBar {

        }
    }
}

#[component]
fn SongView(queue: Signal<QueueManager>) -> Element {
    let mut progress = use_signal(|| 0.0);
    let mut progress_held = use_signal(|| false);

    let skip = move |e: Event<MouseData>| {
        queue.write().skip();
        *CURRENT.write() = queue.read().current();
        progress.set(0.0);
        info!("{:?}", queue.read().current_track());
    };

    let skipback = move |e: Event<MouseData>| {
        queue.write().skipback();
        *CURRENT.write() = queue.read().current();
        progress.set(0.0);
        info!("{:?}", queue.read().current_track());
    };
    
    use_future(move || async move {
        let mut to_add = 0.0;
        loop {
            time::sleep(Duration::from_secs_f64(0.25)).await;
            if !progress_held() {
                *progress.write() += to_add;
                queue.write().progress = progress();
                to_add = 0.0;
            }
            to_add += 0.25;
        }
    });

    rsx! {
        div {
            class: "songview",
            select {
                for queue_info in &queue.read().queues {
                    option {
                        "{queue_info.queue_type}",
                    }
                }
            }
            div {
                class: "imageview",
                img {
                    src: "/trackimage/{CURRENT()}",
                }
            }
            h2 {
                "{queue.read().current_track_title():?}"
            }
            h3 {
                span { 
                    class: "artistspecifier",
                    onclick: move |e| queue.write().add_current_artist_queue(),
                    "{queue.read().current_track_artist():?}" 
                }
                br { }
                span { 
                    class: "albumspecifier",
                    onclick: move |e| queue.write().add_current_album_queue(),
                    "{queue.read().current_track_album():?}" 
                }
                br {}
                span {
                    class: "genresspecifier",
                    if let Some(genres) = queue.read().current_track_genres() {
                        for genre in genres {
                            span {
                                "{genre}"
                            }
                        }
                    }
                }
            }
            div {
                class: "progressrow",
                span {
                    class: "songprogress",
                }
                input {
                    r#type: "range",
                    value: progress,
                    step: 0.25,
                    max: queue.read().player.song_length(),
                    onchange: move |e| {
                        let value = e.value().parse().unwrap();
                        queue.write().player.set_pos(value);
                        progress.set(value)
                    },
                    onmousedown: move |e| progress_held.set(true),
                    onmouseup: move |e| progress_held.set(false),
                }
                span {
                    class: "songlength",
                }
            }
            div {
                class: "buttonrow",
                button {
                    // onclick: move |e| {
                    //     for genre in genres() {
                    //         *genre_weights.write().entry(genre).or_insert(0) += 1;
                    //     }
                    // },
                    class: "like-button",
                    class: "svg-button",
                }
                button {
                    class: "skipprev-button",
                    class: "svg-button",
                    onclick: skipback,
                }
                button {
                    class: "svg-button",
                    onclick: move |e| queue.write().toggle_playing(),
                    background_image: if queue.read().playing() { "url(assets/pause.svg)" } else { "url(assets/play.svg)" },
                }
                button {
                    class: "skip-button",
                    class: "svg-button",
                    onclick: skip,
                }
                button {
                    class: "dislike-button",
                    class: "svg-button",
                }
            }
            div {
                // for genre in genres() {
                //     "{genre} | "
                // }
            }
            div {
                // for i in 0..12.min(matches().len()) {
                //     "{matches()[i].0}, {matches()[i].1}\n"
                // }
            }
            // "{genre_weights:?}"
        }
    }
}

#[component]
pub fn MenuBar() -> Element {
    rsx! {
        div {
            class: "buttonrow",
            button {
                class: "songview-button",
                class: "svg-button",
            }
            button {
                class: "alltracks-button",
                class: "svg-button",
            }
            button {
                class: "album-button",
                class: "svg-button",
            }
            button {
                class: "artist-button",
                class: "svg-button",
            }
            button {
                class: "genres-button",
                class: "svg-button",
            }
            button {
                class: "search-button",
                class: "svg-button",
            }
            button {
                class: "settings-button",
                class: "svg-button",
            }
        }

    }
}

// pub fn get_song(trackid: i32) -> Track {
//     use crate::schema::tracks::dsl::*;
//     use crate::schema::tracks::id;
//
//     tracks
//         .filter(id.eq(trackid))
//         .select(Track::as_select())
//         .load(&mut *DB.write())
//         .expect("Error loading tracks")[0]
//         .clone()
// }
//
// pub fn clear_genre_matches(conn: &mut SqliteConnection) {
//     use crate::schema::genres::dsl::genres;
//
//     diesel::delete(genres).execute(conn);
// }
//
// pub fn find_song_matches(song: &str, genres: &Vec<String>, limit: i32) -> Vec<(String, i32)> {
//     let mut songs = HashMap::new();
//
//     for genre in genres {
//         let genres_songs = load_genre(genre);
//         println!("{:?}, {:?}", genres_songs.len(), genre);
//         for song in genres_songs {
//             *songs.entry(song.file).or_insert(0) += 1;
//         }
//     }
//
//     songs.remove(song);
//
//     let mut songs = songs.into_iter().collect::<Vec<(String, i32)>>();
//     songs.sort_by(|a, b| b.1.cmp(&a.1));
//
//     songs
// }
//
// pub fn track_from_file(file_name: &str) -> Track {
//     use crate::schema::tracks::dsl::*;
//
//     let results = tracks
//         .select(Track::as_select())
//         .filter(file.eq(file_name))
//         .load(&mut *DB.write())
//         .expect("Error loading posts");
//
//     results[0].clone()
// }
//

//
// pub fn load_genre(genre_to_match: &str) -> Vec<Track> {
//     use crate::schema::tracks::dsl::*;
//
//     tracks
//         .filter(genre.like(format!("%{genre_to_match}%")))
//         .load::<Track>(&mut *DB.write())
//         .expect("error")
// }
//
// pub fn create_track(
//     conn: &mut SqliteConnection,
//     file: &str,
//     title: &str,
//     artist: &str,
//     album: &str,
//     genre: &str,
//     date: &str,
//     body: &str,
// ) {
//     use crate::schema::tracks;
//
//     let new_track = NewTrack { file, title, artist, album, genre, date, body };
//
//     diesel::insert_into(tracks::table)
//         .values(&new_track)
//         .returning(Track::as_returning())
//         .on_conflict(tracks::dsl::file)
//         .do_nothing()
//         .execute(conn)
//         .expect("Error saving new track");
// }
//
// pub fn establish_connection() -> SqliteConnection {
//     dotenv().ok();
//
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     SqliteConnection::establish(&database_url)
//         .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
// }
//
fn get_song_files(directory: &str) -> Result<Vec<String>, io::Error> {
    let entries = fs::read_dir(directory)?;

    let mp3_files: Vec<String> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("mp3") {
                path.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(mp3_files)
}

// This was taken from wry's example
async fn get_stream_response(
    asset: &mut (impl tokio::io::AsyncSeek + tokio::io::AsyncRead + Unpin + Send + Sync),
    request: &AssetRequest,
) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error>> {
    // get stream length
    let len = {
        let old_pos = asset.stream_position().await?;
        let len = asset.seek(SeekFrom::End(0)).await?;
        asset.seek(SeekFrom::Start(old_pos)).await?;
        len
    };

    let mut resp = ResponseBuilder::new().header(CONTENT_TYPE, "image/png");

    // if the webview sent a range header, we need to send a 206 in return
    // Actually only macOS and Windows are supported. Linux will ALWAYS return empty headers.
    let http_response = if let Some(range_header) = request.headers().get("range") {
        let not_satisfiable = || {
            ResponseBuilder::new()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(CONTENT_RANGE, format!("bytes */{len}"))
                .body(vec![])
        };

        // parse range header
        let ranges = if let Ok(ranges) = http_range::HttpRange::parse(range_header.to_str()?, len) {
            ranges
                .iter()
                // map the output back to spec range <start-end>, example: 0-499
                .map(|r| (r.start, r.start + r.length - 1))
                .collect::<Vec<_>>()
        } else {
            return Ok(not_satisfiable()?);
        };

        /// The Maximum bytes we send in one range
        const MAX_LEN: u64 = 1000 * 1024;

        if ranges.len() == 1 {
            let &(start, mut end) = ranges.first().unwrap();

            // check if a range is not satisfiable
            //
            // this should be already taken care of by HttpRange::parse
            // but checking here again for extra assurance
            if start >= len || end >= len || end < start {
                return Ok(not_satisfiable()?);
            }

            // adjust end byte for MAX_LEN
            end = start + (end - start).min(len - start).min(MAX_LEN - 1);

            // calculate number of bytes needed to be read
            let bytes_to_read = end + 1 - start;

            // allocate a buf with a suitable capacity
            let mut buf = Vec::with_capacity(bytes_to_read as usize);
            // seek the file to the starting byte
            asset.seek(SeekFrom::Start(start)).await?;
            // read the needed bytes
            asset.take(bytes_to_read).read_to_end(&mut buf).await?;

            resp = resp.header(CONTENT_RANGE, format!("bytes {start}-{end}/{len}"));
            resp = resp.header(CONTENT_LENGTH, end + 1 - start);
            resp = resp.status(StatusCode::PARTIAL_CONTENT);
            resp.body(buf)
        } else {
            let mut buf = Vec::new();
            let ranges = ranges
                .iter()
                .filter_map(|&(start, mut end)| {
                    // filter out unsatisfiable ranges
                    //
                    // this should be already taken care of by HttpRange::parse
                    // but checking here again for extra assurance
                    if start >= len || end >= len || end < start {
                        None
                    } else {
                        // adjust end byte for MAX_LEN
                        end = start + (end - start).min(len - start).min(MAX_LEN - 1);
                        Some((start, end))
                    }
                })
                .collect::<Vec<_>>();

            let boundary = format!("{:x}", rand::random::<u64>());
            let boundary_sep = format!("\r\n--{boundary}\r\n");
            let boundary_closer = format!("\r\n--{boundary}\r\n");

            resp = resp.header(CONTENT_TYPE, format!("multipart/byteranges; boundary={boundary}"));

            for (end, start) in ranges {
                // a new range is being written, write the range boundary
                buf.write_all(boundary_sep.as_bytes()).await?;

                // write the needed headers `Content-Type` and `Content-Range`
                buf.write_all(format!("{CONTENT_TYPE}: image/png\r\n").as_bytes()).await?;
                buf.write_all(format!("{CONTENT_RANGE}: bytes {start}-{end}/{len}\r\n").as_bytes())
                    .await?;

                // write the separator to indicate the start of the range body
                buf.write_all("\r\n".as_bytes()).await?;

                // calculate number of bytes needed to be read
                let bytes_to_read = end + 1 - start;

                let mut local_buf = vec![0_u8; bytes_to_read as usize];
                asset.seek(SeekFrom::Start(start)).await?;
                asset.read_exact(&mut local_buf).await?;
                buf.extend_from_slice(&local_buf);
            }
            // all ranges have been written, write the closing boundary
            buf.write_all(boundary_closer.as_bytes()).await?;

            resp.body(buf)
        }
    } else {
        resp = resp.header(CONTENT_LENGTH, len);
        let mut buf = Vec::with_capacity(len as usize);
        asset.read_to_end(&mut buf).await?;
        resp.body(buf)
    };

    http_response.map_err(Into::into)
}
