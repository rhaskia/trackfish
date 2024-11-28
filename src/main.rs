#![feature(trivial_bounds)]

pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::upsert::*;
use dioxus::prelude::*;
use dotenvy::dotenv;
use id3::Version;
use models::*;
use std::env;
use std::fs;
use std::io;
use std::io::stdin;
use id3::Tag;
use id3::TagLike;
use std::collections::HashMap;
use std::fs::File;

const CURRENT: GlobalSignal<Option<i32>> = GlobalSignal::new(|| Some(2));
const DB: GlobalSignal<SqliteConnection> = GlobalSignal::new(|| establish_connection());
const CURRENT_TRACK: GlobalSignal<Option<Track>> = GlobalSignal::new(|| None);

fn main() {
    let mut conn = establish_connection();
    let songs = get_song_files().unwrap();
    clear_genre_matches(&mut conn);

    for song in songs {
        let mut tag = Tag::read_from_path(song.clone()).unwrap();

        let title = tag.title().unwrap_or_default();
        let artist = tag.artist().unwrap_or_default();
        let album = tag.album().unwrap_or_default();
        let genre = tag.genre().unwrap_or_default().replace("\0", ";");
        let mut year = String::new();
        if let Some(tag_year) = tag.get("Date") {
            year = tag_year.to_string();
            println!("{year}");
        }

        create_track(&mut conn, &song, title, artist, album, &genre, &year, "");
    }

    drop(conn);

    launch(App);
}

#[component]
fn App() -> Element {
    let results = use_signal(|| load_tracks(&mut *DB.write()));

    rsx! {
        SongView {}
    }
}

#[component]
fn SongView() -> Element {
    let current_song = use_memo(|| get_song(CURRENT().unwrap()));
    let genres = use_memo(move || current_song().genre.split(";").map(|s| s.to_string()).collect::<Vec<String>>());
    let matches = use_memo(move || find_song_matches(&current_song().file, &genres(), 0));
    let mut genre_weights = use_signal(|| HashMap::new());

    rsx! {
        h2 {
            "{current_song.read().title}"
        }
        button {
            onclick: move |e| if let Some(ref mut trackno) = *CURRENT.write() {
                let next = track_from_file(&matches.read()[0].0);
                *trackno = next.id.unwrap();
                println!("{:?}", current_song);
            },
            "skip"
        }
        div {
            for genre in genres() {
                "{genre} | "
            }
        }
        div {
            for i in 0..12.min(matches().len()) {
                "{matches()[i].0}, {matches()[i].1}\n"
            }
        }
        button {
            onclick: move |e| {
                for genre in genres() {
                    *genre_weights.write().entry(genre).or_insert(0) += 1;
                }
            },
            "like"
        }
        button {
            "dislike"
        }
        "{genre_weights:?}"
    }
}

pub fn get_song(trackid: i32) -> Track {
    use crate::schema::tracks::id;
    use crate::schema::tracks::dsl::*;

    tracks
        .filter(id.eq(trackid))
        .select(Track::as_select())
        .load(&mut *DB.write())
        .expect("Error loading tracks")
        [0].clone()
}

pub fn clear_genre_matches(conn: &mut SqliteConnection) {
    use crate::schema::genres::dsl::genres;
    
    diesel::delete(genres).execute(conn);
}

pub fn find_song_matches(song: &str, genres: &Vec<String>, limit: i32) -> Vec<(String, i32)> {
    let mut songs = HashMap::new();

    for genre in genres {
        let genres_songs = load_genre(genre);
        println!("{:?}, {:?}", genres_songs.len(), genre);
        for song in genres_songs {
            *songs.entry(song.file).or_insert(0) += 1;
        }
    }

    songs.remove(song);

    let mut songs = songs.into_iter().collect::<Vec<(String, i32)>>();
    songs.sort_by(|a, b| b.1.cmp(&a.1));

    songs
}

pub fn track_from_file(file_name: &str) -> Track {
    use crate::schema::tracks::dsl::*;

    let results = tracks
        .select(Track::as_select())
        .filter(file.eq(file_name))
        .load(&mut *DB.write())
        .expect("Error loading posts");

    results[0].clone()
}

pub fn load_tracks(conn: &mut SqliteConnection) -> Vec<Track> {
    use crate::schema::tracks::dsl::*;

    let results = tracks
        .select(Track::as_select())
        .load(conn)
        .expect("Error loading posts");

    results
}

pub fn load_genre(genre_to_match: &str) -> Vec<Track> {
    use crate::schema::tracks::dsl::*;

    tracks
        .filter(genre.like(format!("%{genre_to_match}%")))
        .load::<Track>(&mut *DB.write()).expect("error")
}

pub fn create_track(
    conn: &mut SqliteConnection,
    file: &str,
    title: &str,
    artist: &str,
    album: &str,
    genre: &str,
    date: &str,
    body: &str,
) {
    use crate::schema::tracks;

    let new_track = NewTrack { file, title, artist, album, genre, date, body };

    diesel::insert_into(tracks::table)
        .values(&new_track)
        .returning(Track::as_returning())
        .on_conflict((tracks::dsl::file)).do_nothing()
        .execute(conn)
        .expect("Error saving new track");
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn get_song_files() -> Result<Vec<String>, io::Error> {
    let directory_path = "E:/music/test";

    let entries = fs::read_dir(directory_path)?;

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
