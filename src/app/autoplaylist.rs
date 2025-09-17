struct AutoPlaylist {
    name: String,
    conditions: Vec<Condition>,
}

enum Condition {
    Artist(String),
    Album(String),
    Genre(String),
    Any(Vec<Condition>),
    All(Vec<Condition>),
    Not(Box<Condition>),
    GreaterThan(Box<Condition>),
    LesserThan(Box<Condition>),
    Year(i64),
    Length(i64), // seconds
}
