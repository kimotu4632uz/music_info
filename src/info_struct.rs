use std::fmt;
use serde::{Serialize, Deserialize};

pub type AddInfo = Vec<(String, String)>;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Track {
    pub title: String,
    pub artist: String,
}

impl Track {
    pub fn new<S: Into<String>>(title: S, artist: S) -> Track {
        Track{ title: title.into(), artist: artist.into() }
    }
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "title: {}\nartist: {}\n", self.title, self.artist)
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Metadata {
    #[serde(skip)]
    pub id: Option<String>,

    pub album: String,
    pub date: u32,
    pub genre: String,
    pub tracks: Vec<Track>,
}

impl Metadata {
    pub fn new<S: Into<String>>(id: Option<S>, album: S, date: u32, genre: S, tracks: Vec<Track>) -> Metadata {
        Metadata{ id: id.map(|x| x.into()), album: album.into(), date: date, genre: genre.into(), tracks: tracks }
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(id) = &self.id {
            write!(f, "id: {}\nalbum: {}\ndate: {}\ngenre: {}\ntracks:\n", id, self.album, self.date, self.genre)?;
        } else {
            write!(f, "album: {}\ndate: {}\ngenre: {}\ntracks:\n", self.album, self.date, self.genre)?;
        }
        for (i, track) in self.tracks.iter().enumerate() {
            write!(f, "  track number: {}\n  track title: {}\n  track artist: {}\n", i + 1, track.title, track.artist)?;
            if i < self.tracks.len() {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}
