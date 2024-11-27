use diesel::prelude::*;
use crate::schema::tracks;

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
