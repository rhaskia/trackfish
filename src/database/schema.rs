// @generated automatically by Diesel CLI.

diesel::table! {
    genres (id) {
        id -> Nullable<Integer>,
        genre1 -> Text,
        genre2 -> Text,
        count -> Integer,
    }
}

diesel::table! {
    listens (id) {
        id -> Nullable<Integer>,
        trackid -> Integer,
        listenstart -> Timestamp,
        listentime -> Integer,
    }
}

diesel::table! {
    tracks (id) {
        id -> Nullable<Integer>,
        file -> Text,
        title -> Text,
        album -> Text,
        artist -> Text,
        genre -> Text,
        date -> Text,
        body -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    genres,
    listens,
    tracks,
);
