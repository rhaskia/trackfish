use diesel::prelude::*;
use crate::schema::tracks;
use crate::schema::genres;

#[derive(QueryableByName, Queryable, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = tracks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Track {
    pub id: Option<i32>,
    pub file: String,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub genre: String,
    pub date: String,
    pub body: String,
}

#[derive(Insertable, PartialEq)]
#[diesel(table_name = tracks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewTrack<'a> {
    pub file: &'a str,
    pub title: &'a str,
    pub album: &'a str,
    pub artist: &'a str,
    pub genre: &'a str,
    pub date: &'a str,
    pub body: &'a str,
}

#[derive(QueryableByName, Queryable, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = genres)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct GenreMatch {
    pub id: Option<i32>,
    pub genre1: String,
    pub genre2: String,
    pub count: i32,
}

#[derive(Insertable, PartialEq)]
#[diesel(table_name = genres)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewGenreMatch<'a> {
    pub genre1: &'a str,
    pub genre2: &'a str,
    pub count: i32,
}
