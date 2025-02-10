use std::fs;
use rusqlite::{params, Connection, Result};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, DefaultHasher, Hasher};
use std::path::Path;
use crate::app::track::Track;
use crate::app::settings::Settings;

pub fn hash_filename(name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish();
}

pub fn init_db() -> Result<Connection> {
    let file = Settings::dir().join("tracks.db");
    let db_exists = file.exists();
    let conn = Connection::open(file)?;

    if !db_exists {
        conn.execute(
            "CREATE TABLE cache (
                file_hash TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                title TEXT NOT NULL,
                album TEXT NOT NULL,
                artists TEXT NOT NULL,
                genres TEXT NOT NULL,
                year TEXT NOT NULL,
                mood TEXT,
                trackno INTEGER NOT NULL,
                len REAL NOT NULL
            )",
            [],
        )?;
    }

    Ok(conn)
}

fn get_from_cache(conn: &Connection, filename: &str) -> Result<Option<CachedItem>> {
    let file_hash = hash_filename(filename);
    let mut stmt = conn.prepare("SELECT * FROM cache WHERE file_hash = ?1")?;
    
    let result = stmt.query_row(params![file_hash], |row| {
        Ok(Track {
            file: row.get(1)?,
            title: row.get(2)?,
            album: row.get(3)?,
            artists: row.get(4)?,
            genre: row.get(5)?,
            mood: row.get(6)?,
            trackno: row.get(7)?,
            year: row.get(8)?,
            len: row.get(9)?,
        })
    }).optional()?;

    Ok(result)
}

fn save_to_cache(conn: &Connection, item: &Track) -> Result<()> {
    let file_hash = hash_filename(&item.filename);
    conn.execute(
        "INSERT INTO cache (file_hash, file_path, title, album, artists, genres, mood, trackno, year, len) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10
        )
        ON CONFLICT(file_hash) DO UPDATE SET 
            filename = excluded.filename,
            size = excluded.size,
            metadata = excluded.metadata",
        params![file_hash, item.file, item.title, item.album, item.artists, items.genres, item.mood, item.trackno, item.year, item.len],
    )?;
    Ok(())
}

impl ToSql for Mood {

}
