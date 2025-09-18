use super::utils::{strip_unnessecary, similar};
use super::track::Track;

pub struct AutoPlaylist {
    name: String,
    conditions: Vec<Condition>,
    // TODO: sort-by?
}

pub enum Condition {
    Is(StrIdentifier, String), // Identifier, Query
    Has(StrIdentifier, String), // Identifier, Query
    Greater(NumIdentifier, i64),
    Lesser(NumIdentifier, i64),
    EqualTo(NumIdentifier, i64),
    Any(Vec<Condition>),
    All(Vec<Condition>),
    Not(Box<Condition>),
    Missing(Identifier),
}

#[derive(Clone, Copy)]
pub enum Identifier {
    Str(StrIdentifier),
    Num(NumIdentifier)
}

#[derive(Clone, Copy)]
pub enum StrIdentifier {
    Title,
    Genre,
    Album,
    Artist,
}

#[derive(Clone, Copy)]
pub enum NumIdentifier {
    Year,
    Length,
    Energy
}


impl Condition {
    pub fn track_qualifies(&self, track: &Track) -> bool {
        use Condition::*;
        use StrIdentifier::*;
        match self {
            Is(ident, value) => match ident {
                StrIdentifier::Title => similar(&track.title, &value),
                StrIdentifier::Artist => track.artists.iter().any(|a| similar(&a, &value)),
                StrIdentifier::Genre => track.genres.iter().any(|g| similar(&g, &value)),
                StrIdentifier::Album => similar(&track.album, &value),
                _ => todo!(),
            },
            Has(ident, value) => match ident {
                StrIdentifier::Title => strip_unnessecary(&track.title).contains(&strip_unnessecary(&value)),
                StrIdentifier::Artist => track.artists.iter().any(|a| strip_unnessecary(&a).contains(&strip_unnessecary(&value))),
                StrIdentifier::Genre => track.genres.iter().any(|g| strip_unnessecary(&g).contains(&strip_unnessecary(&value))),
                StrIdentifier::Album => strip_unnessecary(&track.album).contains(&strip_unnessecary(&value)),
                _ => todo!(),
            },
            Not(cond) => !cond.track_qualifies(&track),
            Missing(ident) => match ident {
                Identifier::Str(str_ident) => match str_ident {
                    StrIdentifier::Title => strip_unnessecary(&track.title).is_empty(),
                    StrIdentifier::Artist => track.artists.is_empty() || track.artists.iter().all(|a| strip_unnessecary(&a).is_empty()),
                    StrIdentifier::Genre => track.genres.is_empty() || track.genres.iter().all(|g| strip_unnessecary(&g).is_empty()),
                    StrIdentifier::Album => strip_unnessecary(&track.title).is_empty(),
                }
                Identifier::Num(num_ident) => match num_ident {
                    _ => todo!(),
                }
            },
            All(conditions) => conditions.iter().all(|a| a.track_qualifies(&track)),
            Any(conditions) => conditions.iter().any(|a| a.track_qualifies(&track)),
            Greater(ident, value) => match ident {
                NumIdentifier::Year => track.year.parse().and_then(|y: i64| Ok(&y > value)).unwrap_or(false),
                NumIdentifier::Length => (track.len as i64) > *value,
                _ => todo!(),
            }
            Lesser(ident, value) => match ident {
                NumIdentifier::Year => track.year.parse().and_then(|y: i64| Ok(&y < value)).unwrap_or(false),
                NumIdentifier::Length => (track.len as i64) < *value,
                _ => todo!(),
            }
            EqualTo(ident, value) => match ident {
                NumIdentifier::Year => track.year.parse().and_then(|y: i64| Ok(&y == value)).unwrap_or(false),
                NumIdentifier::Length => (track.len as i64) == *value,
                _ => todo!(),
            }
        }
    }

    pub fn qualify_tracks(&self, tracks: &Vec<Track>) -> Vec<usize> {
        let mut results = Vec::new();

        for i in 0..tracks.len() {
            if self.track_qualifies(&tracks[i]) {
                results.push(i);
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_track(title: &str, artist: &str) -> Track {
        let mut track = Track::default();
        track.title = title.to_string();
        track.artists = vec![artist.to_string()];
        track
    }

    fn year_track(year: i64) -> Track {
        let mut track = Track::default();
        track.year = year.to_string();
        track
    }

    #[test]
    fn artist_query() {
        let tracks = vec![simple_track("song 1", "john doe"),
            simple_track("song 2", "jimmy bob"),
            simple_track("song 3", "Jane Doe")];

        let query = Condition::Is(StrIdentifier::Artist, "John Doe".to_string());

        assert_eq!(query.qualify_tracks(&tracks), vec![0]);
    }

    #[test]
    fn artist_has_query() {
        let tracks = vec![simple_track("song 1", "john doe"),
            simple_track("song 2", "jimmy bob"),
            simple_track("song 3", "Jane Doe")];

        let query = Condition::Has(StrIdentifier::Artist, "Doe".to_string());

        assert_eq!(query.qualify_tracks(&tracks), vec![0, 2]);
    }

    #[test]
    fn not_query() {
        let tracks = vec![simple_track("song 1", "john doe"),
            simple_track("song 2", "jimmy bob"),
            simple_track("song 3", "Jane Doe")];

        let query = Condition::Not(Box::new((Condition::Is(StrIdentifier::Artist, "John Doe".to_string()))));

        assert_eq!(query.qualify_tracks(&tracks), vec![1,2]);
    }

    #[test]
    fn missing_query() {
        let tracks = vec![simple_track("song 1", "john doe"),
            simple_track("song 2", ""),
            simple_track("song 3", "  "),
            simple_track("song 4", "Jane Doe"),
            simple_track("song 5", "    ")];

        let query = Condition::Missing(Identifier::Str(StrIdentifier::Artist));

        assert_eq!(query.qualify_tracks(&tracks), vec![1,2,4]);
    }

    #[test]
    fn any_all_query() {
        let tracks = vec![simple_track("song 1", "john doe"),
            simple_track("song 2", "Jane Doe"),
            simple_track("track 3", "Jimmy Bob"),
            simple_track("song 4", "Jane Doe"),
            simple_track("track 5", "John Doe")];

        let query = Condition::All(vec![
            Condition::Is(StrIdentifier::Artist, "John Doe".to_string()),
            Condition::Has(StrIdentifier::Title, "track".to_string())]);

        assert_eq!(query.qualify_tracks(&tracks), vec![4]);

        let query = Condition::Any(vec![
            Condition::Is(StrIdentifier::Artist, "John Doe".to_string()),
            Condition::Has(StrIdentifier::Title, "track".to_string())]);

        assert_eq!(query.qualify_tracks(&tracks), vec![0,2,4]);
    }

    #[test]
    fn num_query() {
        let tracks = vec![year_track(1980),
            year_track(1976),
            year_track(1969),
            year_track(2010),
            year_track(2024)];

        let query = Condition::Greater(NumIdentifier::Year, 1980);

        assert_eq!(query.qualify_tracks(&tracks), vec![3,4]);

        let query = Condition::Lesser(NumIdentifier::Year, 1980);

        assert_eq!(query.qualify_tracks(&tracks), vec![1,2]);

        let query = Condition::EqualTo(NumIdentifier::Year, 1980);

        assert_eq!(query.qualify_tracks(&tracks), vec![0]);
    }
}
