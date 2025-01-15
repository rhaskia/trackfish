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

