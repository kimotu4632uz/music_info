use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CoverArtArchive {
    pub count: i32,
    pub artwork: bool,
    pub front: bool,
    pub back: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Release {
    pub id: String,
    pub title: String,
    pub date: String,
    pub genres: Vec<String>,
    pub cover_art_archive: CoverArtArchive,

    pub barcode: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArtistCredit {
    pub name: String,
    pub joinphrase: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Recording {
    pub title: String,
    pub artist_credit: Vec<ArtistCredit>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cover {
    pub images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub front: bool,
    pub back: bool,
    pub image: String,
}
