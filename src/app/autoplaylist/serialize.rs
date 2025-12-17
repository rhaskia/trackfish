use log::info;

use super::{AutoPlaylist, Condition};
use std::path::PathBuf;

impl AutoPlaylist {
    /// Gets the directory for caching information
    /// Probably should have a sort of thing to make it so you can save them to the main dir
    pub fn dir(&self) -> PathBuf {
        let dir = if cfg!(target_os = "android") {
            cache_dir::get_cache_dir().unwrap()
        } else {
            dirs::config_dir().unwrap().join("trackfish/")
        };
        dir.join(format!("{}.auto", self.name))
    }

    pub fn save(&self) {
        info!("Saving autoplaylist {} at path {:?}", self.name, self.dir());
        let _ = std::fs::write(self.dir(), self.conditions.serialize()); 
    }

    pub fn serialize(&self) -> String {
        self.conditions.serialize() 
    }
}

impl Condition {
    pub fn serialize(&self) -> String {
        match self {
            Condition::StrCondition(ident, op, value) => format!("{ident} {op} {value:?}"),
            Condition::NumCondition(ident, op, value) => format!("{ident} {op} {value}"),
            Condition::TimeCondition(ident, op, value) => format!("{ident} {op} {value}"),
            Condition::Any(conditions) => format!("ANY({})", conditions.iter().map(|c| c.serialize()).collect::<Vec<String>>().join(", ")),
            Condition::All(conditions) => format!("ALL({})", conditions.iter().map(|c| c.serialize()).collect::<Vec<String>>().join(", ")),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::app::autoplaylist::{StrOperator, StrIdentifier};

    use super::*;

    #[test]
    pub fn basic_serialization() {
        assert_eq!(Condition::StrCondition(StrIdentifier::Title, StrOperator::Is, "Track".to_string()).serialize(), "Title IS \"Track\"".to_string());
        assert_eq!(Condition::StrCondition(StrIdentifier::Title, StrOperator::Is, "Track with space".to_string()).serialize(), "Title IS \"Track with space\"".to_string());
    }

    #[test]
    pub fn list_serialize() {
        let cond = Condition::All(vec![Condition::StrCondition(StrIdentifier::Title, StrOperator::Is, "Track".to_string()), Condition::StrCondition(StrIdentifier::Album, StrOperator::Has, "Album".to_string())]);

        assert_eq!(cond.serialize(), "ALL(Title IS \"Track\", Album HAS \"Album\")");
    }
}
