use log::info;

use super::{AutoPlaylist, Condition, StrIdentifier, NumIdentifier, Identifier};
use std::path::PathBuf;

impl AutoPlaylist {
    /// Gets the directory for caching information
    /// Probably should have a sort of thing to make it so you can save them to the main dir
    pub fn dir() -> PathBuf {
        if cfg!(target_os = "android") {
            cache_dir::get_cache_dir().unwrap()
        } else {
            dirs::config_dir().unwrap().join("trackfish/")
        }
    }

    pub fn save(&self) {
        let dir = Self::dir().join(format!("{}.auto", self.name));
        info!("Saving autoplaylist {} at path {dir:?}", self.name);
        std::fs::write(dir, self.conditions.serialize()); 
    }

    pub fn serialize(&self) -> String {
        self.conditions.serialize() 
    }
}

impl Condition {
    pub fn serialize(&self) -> String {
        match self {
            Condition::Is(ident, value) => format!("{ident} IS {value:?}"),
            Condition::Has(ident, value) => format!("{ident} HAS {value:?}"),
            Condition::Greater(ident, value) => format!("{ident} GREATER {value}"),
            Condition::Lesser(ident, value) => format!("{ident} GREATER {value}"),
            Condition::EqualTo(ident, value) => format!("{ident} EQUALS {value}"),
            Condition::Any(conditions) => format!("ANY({})", conditions.iter().map(|c| c.serialize()).collect::<Vec<String>>().join(", ")),
            Condition::All(conditions) => format!("ALL({})", conditions.iter().map(|c| c.serialize()).collect::<Vec<String>>().join(", ")),
            Condition::Not(maybe_cond) => match maybe_cond { 
                Some(cond) => format!("NOT {}", cond.serialize()),
                None => format!("NOT ?"),
            }
            Condition::Missing(ident) => format!("MISSING {ident}"),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn basic_serialization() {
        assert_eq!(Condition::Is(StrIdentifier::Title, "Track".to_string()).serialize(), "Title IS \"Track\"".to_string());
        assert_eq!(Condition::Is(StrIdentifier::Title, "Track with space".to_string()).serialize(), "Title IS \"Track with space\"".to_string());
    }

    #[test]
    pub fn list_serialize() {
        let cond = Condition::All(vec![Condition::Is(StrIdentifier::Title, "Track".to_string()), Condition::Has(StrIdentifier::Album, "Album".to_string())]);

        assert_eq!(cond.serialize(), "ALL(Title IS \"Track\", Album HAS \"Album\")");
    }
}
