use super::utils::{strip_unnessecary, similar};
use super::track::Track;
use std::fmt::Display;
use std::ops::{Index, IndexMut};
use std::str::FromStr;
use strum_macros::EnumString;
use strum_macros::Display;
use log::info;

/// Autoplaylist struct
/// Can be used to get a set of tracks that fit a certain conditions or set of conditions
#[derive(PartialEq, Clone)]
pub struct AutoPlaylist {
    pub name: String,
    pub conditions: Condition,
    // TODO: sort-by?
}

impl AutoPlaylist {
    pub fn new(name: String) -> Self {
        Self { name, conditions: Condition::All(vec![
            Condition::Is(StrIdentifier::Title, "Test".to_string()),
            Condition::Has(StrIdentifier::Title, "Test".to_string()),
            Condition::Greater(NumIdentifier::Year, 1970),
            Condition::Any(vec![
                Condition::EqualTo(NumIdentifier::Year, 1980),
                Condition::Lesser(NumIdentifier::Year, 1990),
            ]),
            Condition::Not(Some(Box::new(Condition::Is(StrIdentifier::Title, "Random Title".to_string())))),
            Condition::Missing(Identifier::Str(StrIdentifier::Title)),
        ]) }
    }
}

/// Condition enum for autoplaylists 
#[derive(PartialEq, Clone, Debug)]
pub enum Condition {
    Is(StrIdentifier, String), // Identifier, Query
    Has(StrIdentifier, String), // Identifier, Query
    Greater(NumIdentifier, i64),
    Lesser(NumIdentifier, i64),
    EqualTo(NumIdentifier, i64),
    Any(Vec<Condition>),
    All(Vec<Condition>),
    Not(Option<Box<Condition>>),
    Missing(Identifier),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Identifier {
    Str(StrIdentifier),
    Num(NumIdentifier)
}

#[derive(Clone, Copy, PartialEq, Debug, EnumString, Display)]
pub enum StrIdentifier {
    Title,
    Genre,
    Album,
    Artist,
}

#[derive(Clone, Copy, PartialEq, Debug, EnumString, Display)]
pub enum NumIdentifier {
    Year,
    Length,
    Energy
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Str(s) => s.fmt(f),
            Identifier::Num(n) => n.fmt(f),
        } 
    }
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
            },
            Has(ident, value) => match ident {
                StrIdentifier::Title => strip_unnessecary(&track.title).contains(&strip_unnessecary(&value)),
                StrIdentifier::Artist => track.artists.iter().any(|a| strip_unnessecary(&a).contains(&strip_unnessecary(&value))),
                StrIdentifier::Genre => track.genres.iter().any(|g| strip_unnessecary(&g).contains(&strip_unnessecary(&value))),
                StrIdentifier::Album => strip_unnessecary(&track.album).contains(&strip_unnessecary(&value)),
            },
            Not(cond) => !cond.as_ref().and_then(|c| Some(c.track_qualifies(&track))).unwrap_or(false),
            Missing(ident) => match ident {
                Identifier::Str(str_ident) => match str_ident {
                    StrIdentifier::Title => strip_unnessecary(&track.title).is_empty(),
                    StrIdentifier::Artist => track.artists.is_empty() || track.artists.iter().all(|a| strip_unnessecary(&a).is_empty()),
                    StrIdentifier::Genre => track.genres.is_empty() || track.genres.iter().all(|g| strip_unnessecary(&g).is_empty()),
                    StrIdentifier::Album => strip_unnessecary(&track.title).is_empty(),
                }
                Identifier::Num(num_ident) => match num_ident {
                    NumIdentifier::Year => strip_unnessecary(&track.year).is_empty(),
                    NumIdentifier::Length => track.len == 0.0,
                    NumIdentifier::Energy => false,
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

    pub fn set_ident(&mut self, ident: String) {
        info!("{ident}");
        match self {
            Condition::Is(ref mut i, _) => *i = StrIdentifier::from_str(&ident).unwrap(), 
            Condition::Has(ref mut i, _) => *i = StrIdentifier::from_str(&ident).unwrap(), 
            Condition::Greater(ref mut i, _) => *i = NumIdentifier::from_str(&ident).unwrap(), 
            Condition::Lesser(ref mut i, _) => *i = NumIdentifier::from_str(&ident).unwrap(), 
            Condition::EqualTo(ref mut i, _) => *i = NumIdentifier::from_str(&ident).unwrap(), 
            Condition::Missing(ref mut i) => *i = if let Ok(str) = StrIdentifier::from_str(&ident) {
                Identifier::Str(str)
            } else {
                Identifier::Num(NumIdentifier::from_str(&ident).unwrap())
            },
            _ => {}
        }
    } 

    pub fn set_value(&mut self, value: String) {
        match self {
            Condition::Is(_, ref mut v) => *v = value.into(), 
            Condition::Has(_, ref mut v) => *v = value.into(), 
            Condition::Lesser(_, ref mut v) => *v = value.parse().unwrap_or(0),
            Condition::Greater(_, ref mut v) => *v = value.parse().unwrap_or(0),
            Condition::EqualTo(_, ref mut v) => *v = value.parse().unwrap_or(0),
            _ => {}
        }
    } 

    pub fn add(&mut self, condition: Condition) {
        match self {
            Condition::All(ref mut all) => all.push(condition),
            Condition::Any(ref mut any) => any.push(condition),
            Condition::Not(ref mut not) => *not = Some(Box::new(condition)),
            _ => {}
        }
    }

    pub fn remove(&mut self, index: usize) {
        match self {
            Condition::All(ref mut all) => { all.remove(index); },
            Condition::Any(ref mut any) => { any.remove(index); },
            Condition::Not(ref mut not) => *not = None,
            _ => {}
        }
    }
}

impl Index<Vec<usize>> for Condition {
    type Output = Self;

    fn index(&self, mut index: Vec<usize>) -> &Self::Output {
        if index.len() == 0 {
            return &self;
        }

        let first_index = index.remove(0);

        let inner = match self {
            Condition::Any(conditions) => &conditions[first_index],
            Condition::All(conditions) => &conditions[first_index],
            Condition::Not(condition) => if first_index == 0 { &condition.as_ref().unwrap() } else { panic!("Index {first_index} out of range for Condition::Not") },
            _ => panic!("{self:?} does not support indexing"),
        };

        if index.len() == 0 {
            return inner;
        } else {
            &inner[index]
        }
    }
}

impl IndexMut<Vec<usize>> for Condition {
    fn index_mut(&mut self, mut index: Vec<usize>) -> &mut Self::Output {
        if index.len() == 0 {
            return self;
        }

        let first_index = index.remove(0);

        let inner = match self {
            Condition::Any(conditions) => &mut conditions[first_index],
            Condition::All(conditions) => &mut conditions[first_index],
            Condition::Not(condition) => if first_index == 0 { condition.as_mut().unwrap() } else { panic!("Index {first_index} out of range for Condition::Not") },
            _ => panic!("{self:?} does not support indexing"),
        };

        if index.len() == 0 {
            return inner;
        } else {
            &mut inner[index]
        }
    }
}

impl Into<StrIdentifier> for String {
    fn into(self) -> StrIdentifier {
        match self.as_str() {
            "Title" => StrIdentifier::Title,
            "Artist" => StrIdentifier::Artist,
            "Album" => StrIdentifier::Album,
            "Genre" => StrIdentifier::Genre,
            _ => unreachable!(),
        }
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
