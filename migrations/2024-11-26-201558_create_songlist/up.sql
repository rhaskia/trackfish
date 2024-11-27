CREATE TABLE tracks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    album TEXT NOT NULL,
    artist TEXT NOT NULL,
    genre TEXT NOT NULL,
    date TEXT NOT NULL,
    body TEXT NOT NULL
);

CREATE TABLE listens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trackid INTEGER NOT NULL,
    listenstart DATETIME NOT NULL,
    listentime INTEGER NOT NULL
);
