use log::info;
use rusqlite::{
    params,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
    Connection, OptionalExtension, Result, Row, ToSql,
};

use crate::app::settings::Settings;
use crate::app::track::{Mood, Track, TrackInfo};
use ndarray::Array1;

pub fn hash_filename(name: &str) -> String {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish().to_string()
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn table_outdated(conn: &Connection, table: &str) -> bool {
    let result = match conn.prepare(&format!(
        "
        SELECT * FROM {table}
    "
    )) {
        Ok(res) => res,
        Err(_) => return false,
    };

    let columns_needed = vec![
        "file_hash",
        "genre_space",
        "spectral",
        "chroma",
        "energy",
        "key",
        "bpm",
        "zcr",
    ];

    if columns_needed.len() != result.column_count() {
        return false;
    }

    for (a, b) in result.column_names().into_iter().zip(columns_needed) {
        if a != b {
            info!("column {a} does not match needed column {b}");
            return true;
        }
    }

    false
}

pub fn init_db() -> Result<Connection> {
    let file = Settings::dir().join("tracks.db");
    let db_exists = file.exists();
    info!("Database exists at {file:?}: {db_exists}");

    let conn = Connection::open(file)?;

    if table_outdated(&conn, "weights") {
        conn.execute("DROP TABLE weights", params![])
            .expect("Could not drop table weights");
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tracks (
            file_hash TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            title TEXT NOT NULL,
            album TEXT NOT NULL,
            artists TEXT NOT NULL,
            genres TEXT NOT NULL,
            mood TEXT,
            trackno INTEGER NOT NULL,
            year TEXT NOT NULL,
            len REAL NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS weights (
            file_hash TEXT PRIMARY KEY,
            genre_space BLOB,
            mfcc BLOB,
            spectral BLOB,
            chroma BLOB,
            energy FLOAT,
            key INT,
            bpm FLOAT,
            zcr FLOAT
        ) ",
        [],
    )?;

    Ok(conn)
}

pub fn get_from_cache(conn: &Connection, filename: &str) -> Result<Option<Track>> {
    let file_hash = hash_filename(filename);
    let mut stmt = conn.prepare("SELECT * FROM tracks WHERE file_hash = ?1")?;

    let result = stmt
        .query_row(params![file_hash], |row| {
            let artists_raw: String = row.get(4)?;
            let artists = artists_raw.split(";").map(|s| s.to_string()).collect();
            let genres_raw: String = row.get(5)?;
            let genres = genres_raw.split(";").map(|s| s.to_string()).collect();
            let mood_raw: Option<String> = row.get(6)?;
            let mood = match mood_raw {
                Some(text) => Some(string_to_mood(&text)),
                None => None,
            };

            Ok(Track {
                file: row.get(1)?,
                title: row.get(2)?,
                album: row.get(3)?,
                artists,
                genres,
                mood,
                trackno: row.get(7)?,
                year: row.get(8)?,
                len: row.get(9)?,
            })
        })
        .optional()?;

    Ok(result)
}

fn to_blob(array: &Array1<f32>) -> Vec<u8> {
    array.iter().map(|f| f.to_le_bytes()).flatten().collect()
}

pub fn save_track_weights(conn: &Connection, track: &str, weights: &TrackInfo) -> Result<()> {
    let file_hash = hash_filename(track);
    let genre_blob: Vec<u8> = to_blob(&weights.genre_space);
    let mfcc_blob: Vec<u8> = to_blob(&weights.mfcc);
    let chroma_blob: Vec<u8> = to_blob(&weights.chroma);
    let spectral_blob: Vec<u8> = to_blob(&weights.spectral);

    conn.execute(
        "INSERT OR REPLACE INTO weights 
        (file_hash,
         genre_space,
         mfcc,
         chroma,
         spectral,
         energy,
         key,
         bpm,
         zcr) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            file_hash,
            genre_blob,
            mfcc_blob,
            chroma_blob,
            spectral_blob,
            weights.energy,
            weights.key,
            weights.bpm,
            weights.zcr
        ],
    )?;

    Ok(())
}

pub fn blob_to_array(blob: Vec<u8>) -> Array1<f32> {
    let mut weights = vec![];
    let mut raw = [0; 4];
    for i in 0..(blob.len() / 4) {
        raw.copy_from_slice(&blob[i * 4..i * 4 + 4]);
        weights.push(f32::from_le_bytes(raw));
    }

    Array1::from_vec(weights)
}

pub fn cached_weight(conn: &Connection, track: &str) -> Result<TrackInfo> {
    let file_hash = hash_filename(track);
    let mut stmt = conn.prepare("SELECT * FROM weights WHERE file_hash = ?1")?;

    stmt.query_row(params![file_hash], |row| row_to_weights(&row))
}

pub fn row_to_weights(row: &Row) -> Result<TrackInfo> {
    let genre_space = blob_to_array(row.get(1)?);
    let mfcc = blob_to_array(row.get(2)?);
    let chroma = blob_to_array(row.get(3)?);
    let spectral = blob_to_array(row.get(4)?);
    let energy = row.get(5)?;
    let key = row.get(6)?;
    let bpm = row.get(7)?;
    let zcr = row.get(8)?;

    Ok(TrackInfo {
        genre_space,
        mfcc,
        chroma,
        spectral,
        energy,
        key,
        bpm,
        zcr,
    })
}

pub fn save_to_cache(conn: &Connection, item: &Track) -> Result<()> {
    let file_hash = hash_filename(&item.file);
    conn.execute(
        "INSERT OR REPLACE INTO tracks (file_hash, file_path, title, album, artists, genres, mood, trackno, year, len) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            file_hash,
            item.file,
            item.title,
            item.album,
            item.artists.join(";"),
            item.genres.join(";"),
            item.mood,
            item.trackno,
            item.year,
            item.len
        ],
    )?;
    Ok(())
}

impl ToSql for Mood {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        let text: Vec<&str> = self
            .to_vec()
            .into_iter()
            .map(|b| if b { "Y" } else { "N" })
            .collect();
        Ok(ToSqlOutput::Owned(Value::Text(text.join("").to_string())))
    }
}

impl FromSql for Mood {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let text = if let ValueRef::Text(text) = value {
            text
        } else {
            return FromSqlResult::Err(FromSqlError::InvalidType);
        };

        let bools = text
            .iter()
            .map(|c| if *c == b'Y' { true } else { false })
            .collect();

        FromSqlResult::Ok(Self::from_vec(bools))
    }
}

fn string_to_mood(s: &str) -> Mood {
    let bools = s
        .as_bytes()
        .iter()
        .map(|c| if *c == b'Y' { true } else { false })
        .collect();
    Mood::from_vec(bools)
}
