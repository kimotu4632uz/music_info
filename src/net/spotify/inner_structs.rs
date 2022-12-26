use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Artist {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub height: i32,
    pub width: i32,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub track_number: i32,
    pub name: String,
    pub artists: Vec<Artist>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tracks {
    pub items: Vec<Item>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Album {
    pub genres: Vec<String>,
    pub images: Vec<Image>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
    pub tracks: Tracks,
    pub external_ids: serde_json::Map<String, serde_json::Value>,
}
