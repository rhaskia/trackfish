use super::utils::{strip_unnessecary, similar};
use super::track::Track;
use std::fmt::Display;
use std::ops::{Index, IndexMut};
use std::str::FromStr;
use strum_macros::EnumString;
use strum_macros::Display;
use std::path::PathBuf;
use log::info;

pub mod serialize;
pub mod deserialize;

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
        Self { name, conditions: Condition::All(vec![]) }
    }

    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(&path)?;
        let name = path.file_stem().unwrap_or_default().to_str().unwrap_or_default().to_string();

        Ok(AutoPlaylist {
            name,
            conditions: Condition::deserialize(s)?
        })
    }
}

/// Condition enum for autoplaylists 
#[derive(PartialEq, Clone, Debug)]
pub enum Condition {
    StrCondition(StrIdentifier, StrOperator, String), // Identifier, Query
    NumCondition(NumIdentifier, NumOperator, i64), // Identifier, Query
    TimeCondition(TimeIdentifier, NumOperator, i64), // Identifier, Query
    Any(Vec<Condition>),
    All(Vec<Condition>),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Identifier {
    Str(StrIdentifier),
    Num(NumIdentifier),
    Time(TimeIdentifier)
}

#[derive(Clone, Copy, PartialEq, Debug, EnumString, Display)]
pub enum StrIdentifier {
    #[strum(ascii_case_insensitive)]
    Title,
    #[strum(ascii_case_insensitive)]
    Genre,
    #[strum(ascii_case_insensitive)]
    Album,
    #[strum(ascii_case_insensitive)]
    Artist,
}

#[derive(Clone, Copy, PartialEq, Debug, EnumString, Display)]
pub enum NumIdentifier {
    #[strum(ascii_case_insensitive)]
    Year,
    #[strum(ascii_case_insensitive)]
    Energy
}

#[derive(Clone, Copy, PartialEq, Debug, EnumString, Display)]
pub enum TimeIdentifier {
    #[strum(ascii_case_insensitive)]
    Length,
}

#[derive(Clone, Copy, PartialEq, Debug, EnumString, Display)]
pub enum NumOperator {
    #[strum(ascii_case_insensitive)]
    Greater,
    #[strum(ascii_case_insensitive)]
    Lesser,
    #[strum(ascii_case_insensitive)]
    Equals,
    #[strum(ascii_case_insensitive)]
    NotEqual,
    #[strum(ascii_case_insensitive)]
    Missing
}

#[derive(Clone, Copy, PartialEq, Debug, EnumString, Display)]
pub enum StrOperator {
    #[strum(ascii_case_insensitive)]
    Is,
    #[strum(ascii_case_insensitive)]
    IsNot,
    #[strum(ascii_case_insensitive)]
    Has,
    #[strum(ascii_case_insensitive)]
    HasNot,
    #[strum(ascii_case_insensitive)]
    Missing
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Str(s) => s.fmt(f),
            Identifier::Num(n) => n.fmt(f),
            Identifier::Time(t) => t.fmt(f),
        } 
    }
}

impl Condition {
    pub fn track_qualifies(&self, track: &Track) -> bool {
        use Condition::*;
        use StrIdentifier::*;
        match self {
            All(conditions) => conditions.iter().all(|a| a.track_qualifies(&track)),
            Any(conditions) => conditions.iter().any(|a| a.track_qualifies(&track)),
            StrCondition(ident, op, value) => {
                let actual_value = match ident {
                    Title => vec![track.title.clone()],
                    Genre => track.genres.clone(),
                    Album => vec![track.album.clone()],
                    Artist => track.artists.clone(),
                };

                match op {
                    StrOperator::Is => actual_value.iter().any(|v| similar(&v, &value)),
                    StrOperator::IsNot => !actual_value.iter().any(|v| similar(&v, &value)),
                    StrOperator::Has => actual_value.iter().any(|v| strip_unnessecary(&v).contains(&strip_unnessecary(&value))),
                    StrOperator::HasNot => !actual_value.iter().any(|v| strip_unnessecary(&v).contains(&strip_unnessecary(&value))),
                    StrOperator::Missing => actual_value.is_empty() || actual_value.iter().all(|v| v.is_empty()),
                }
            }
            NumCondition(ident, op, value) => {
                let actual_value = match ident {
                    NumIdentifier::Year => track.year.parse::<i64>().unwrap_or(0),
                    NumIdentifier::Energy => 1,
                };

                match op {
                    NumOperator::Greater => actual_value > *value,
                    NumOperator::Lesser => actual_value < *value,
                    NumOperator::Equals => actual_value == *value,
                    NumOperator::NotEqual => actual_value != *value,
                    NumOperator::Missing => actual_value == 0, // Assume missing to be set to 0 for
                                                               // things such as years
                }
            }
            TimeCondition(ident, op, value) => {
                let actual_value = match ident {
                    TimeIdentifier::Length => track.len,
                } as i64;

                match op {
                    NumOperator::Greater => actual_value > *value,
                    NumOperator::Lesser => actual_value < *value,
                    NumOperator::Equals => actual_value == *value,
                    NumOperator::NotEqual => actual_value != *value,
                    NumOperator::Missing => actual_value == 0,
                }
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
        if let Ok(str_ident) = StrIdentifier::from_str(&ident) {
            match self {
                Condition::StrCondition(ref mut i, _, _) => *i = str_ident,
                _ => *self = Condition::StrCondition(str_ident, StrOperator::Is, String::new()),
            }
        }

        if let Ok(num_ident) = NumIdentifier::from_str(&ident) {
            match self {
                Condition::NumCondition(ref mut i, _, _) => *i = num_ident,
                _ => *self = Condition::NumCondition(num_ident, NumOperator::Greater, 0),
            }
        }

        if let Ok(time_ident) = TimeIdentifier::from_str(&ident) {
            match self {
                Condition::TimeCondition(ref mut i, _, _) => *i = time_ident,
                _ => *self = Condition::TimeCondition(time_ident, NumOperator::Greater, 0),
            }
        }
    } 

    pub fn set_op(&mut self, op: String) {
        match self {
            Condition::StrCondition(_, ref mut o, _) => *o = StrOperator::from_str(&op).unwrap(),
            Condition::NumCondition(_, ref mut o, _)=> *o = NumOperator::from_str(&op).unwrap(),
            Condition::TimeCondition(_, ref mut o, _) => *o = NumOperator::from_str(&op).unwrap(),
            _ => {}
        }
    } 

    pub fn set_value(&mut self, value: String) {
        match self {
            Condition::StrCondition(_, _, ref mut v) => *v = value,
            Condition::NumCondition(_, _, ref mut v) => *v = value.parse::<i64>().unwrap_or(0),
            Condition::TimeCondition(_, _, ref mut v) => *v = value.parse::<i64>().unwrap_or(0),
            _ => {}
        }
    } 

    pub fn add(&mut self, condition: Condition) {
        match self {
            Condition::All(ref mut all) => all.push(condition),
            Condition::Any(ref mut any) => any.push(condition),
            _ => {}
        }
    }

    pub fn remove(&mut self, index: usize) {
        match self {
            Condition::All(ref mut all) => { all.remove(index); },
            Condition::Any(ref mut any) => { any.remove(index); },
            _ => {}
        }
    }

    pub fn toggle_group(&mut self) {
        match self {
            Condition::Any(conditions) => *self = Condition::All(conditions.clone()),
            Condition::All(conditions) => *self = Condition::Any(conditions.clone()),
            _ => {}
        }
    }

    pub fn is_all_or_any(&self) -> bool {
        match self {
            Condition::Any(_) | Condition::All(_) => true,
            _ => false
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

        let query = Condition::StrCondition(StrIdentifier::Artist, StrOperator::Is, "John Doe".to_string());

        assert_eq!(query.qualify_tracks(&tracks), vec![0]);
    }

    #[test]
    fn artist_has_query() {
        let tracks = vec![simple_track("song 1", "john doe"),
            simple_track("song 2", "jimmy bob"),
            simple_track("song 3", "Jane Doe")];

        let query = Condition::StrCondition(StrIdentifier::Artist, StrOperator::Has, "Doe".to_string());

        assert_eq!(query.qualify_tracks(&tracks), vec![0, 2]);
    }

    #[test]
    fn not_query() {
        let tracks = vec![simple_track("song 1", "john doe"),
            simple_track("song 2", "jimmy bob"),
            simple_track("song 3", "Jane Doe")];

        let query = Condition::StrCondition(StrIdentifier::Artist, StrOperator::IsNot, "John Doe".to_string());

        assert_eq!(query.qualify_tracks(&tracks), vec![1,2]);
    }

    #[test]
    fn missing_query() {
        let tracks = vec![simple_track("song 1", "john doe"),
            simple_track("song 2", ""),
            simple_track("song 3", "  "),
            simple_track("song 4", "Jane Doe"),
            simple_track("song 5", "    ")];

        let query = Condition::StrCondition(StrIdentifier::Artist, StrOperator::Missing, String::new());

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
            Condition::StrCondition(StrIdentifier::Artist, StrOperator::Is, "John Doe".to_string()),
            Condition::StrCondition(StrIdentifier::Title, StrOperator::Has, "track".to_string())]);

        assert_eq!(query.qualify_tracks(&tracks), vec![4]);

        let query = Condition::Any(vec![
            Condition::StrCondition(StrIdentifier::Artist, StrOperator::Is, "John Doe".to_string()),
            Condition::StrCondition(StrIdentifier::Title, StrOperator::Has, "track".to_string())]);

        assert_eq!(query.qualify_tracks(&tracks), vec![0,2,4]);
    }

    #[test]
    fn num_query() {
        let tracks = vec![year_track(1980),
            year_track(1976),
            year_track(1969),
            year_track(2010),
            year_track(2024)];

        let query = Condition::NumCondition(NumIdentifier::Year, NumOperator::Greater, 1980);

        assert_eq!(query.qualify_tracks(&tracks), vec![3,4]);

        let query = Condition::NumCondition(NumIdentifier::Year, NumOperator::Lesser, 1980);

        assert_eq!(query.qualify_tracks(&tracks), vec![1,2]);

        let query = Condition::NumCondition(NumIdentifier::Year, NumOperator::Equals, 1980);

        assert_eq!(query.qualify_tracks(&tracks), vec![0]);
    }
}
