use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    pub title: String,
    pub artist: String,
}

impl Track {
    pub fn new<S: Into<String>>(title: S, artist: S) -> Track {
        Track{ title: title.into(), artist: artist.into() }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub album: String,
    pub date: i32,
    pub genre: String,
    pub tracks: Vec<Track>,
}

impl Metadata {
    pub fn new<S: Into<String>>(album: S, date: i32, genre: S, tracks: Vec<Track>) -> Metadata {
        Metadata{ album: album.into(), date: date, genre: genre.into(), tracks: tracks }
    }
}