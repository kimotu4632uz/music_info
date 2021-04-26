use std::str::FromStr;
use serde::{Serialize, Deserialize};
use crate::traits::FetchMeta;
use crate::info_struct::*;

pub struct MusicBrainz {
    client: ureq::Agent,
}

#[derive(Serialize, Deserialize, Debug)]
struct Release {
    id: String,
    barcode: Option<String>,
    title: String,
    date: String,
    genres: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ArtistCredit {
    name: String,
    joinphrase: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Recording {
    title: String,
    
    #[serde(rename="artist-credit")]
    artist_credit: Vec<ArtistCredit>,
}

impl MusicBrainz {
    fn new() -> MusicBrainz {
        let client = ureq::agent();
        MusicBrainz { client: client }
    }

    fn get_mb(&self, url: &str, query: &[(&str, &str)]) -> anyhow::Result<serde_json::Value> {
        let mut client = self.client
            .get(url)
            .query("fmt", "json");
        
        for (key, val) in query {
            client = client.query(key, val);
        }

        let json: serde_json::Value = client
            .call()?
            .into_json()?;

        if let Some(err) = json.get("error") {
            Err(anyhow::anyhow!("Failed to query for musicbrainz: {}", err))
        } else {
            Ok(json)
        }
    }
}


impl FetchMeta for MusicBrainz {
    fn query(&self, query: &str) -> anyhow::Result<Vec<Metadata>> {
        let json = self.get_mb("http://musicbrainz.org/ws/2/release/", &[("query", query)])?;

        let release_ids = json["releases"].as_array().unwrap().into_iter().map(|e| e["id"].as_str().unwrap());
        let mut result = Vec::with_capacity(release_ids.len());

        for release in release_ids {
            result.push(self.fetch(release)?);
        }

        Ok(result)
    }

    fn fetch(&self, id: &str) -> anyhow::Result<Metadata> {
        let release_json = self.get_mb(&format!("http://musicbrainz.org/ws/2/release/{}", id), &[("inc", "recordings+genres")])?;
        let recording_ids = release_json["media"][0]["tracks"].as_array().unwrap().into_iter().map(|e| e["recording"]["id"].as_str().unwrap());


        let mut tracks = Vec::with_capacity(recording_ids.len());
        for recording_id in recording_ids {
            let recording_json = self.get_mb(&format!("http://musicbrainz.org/ws/2/recording/{}", recording_id), &[("inc", "artists")])?;
            let recording: Recording = serde_json::from_value(recording_json)?;

            let artist = recording.artist_credit.iter().fold(String::new(), |acc, e| acc + &e.name + &e.joinphrase);

            tracks.push(Track::new(recording.title, artist));
        }

        let release: Release = serde_json::from_value(release_json)?;
        let date = release.date.split("-").next().unwrap();
        let date = i32::from_str(date)?;

        Ok(Metadata::new(release.title, date, release.genres.first().map(|e| e.to_string()).unwrap_or_default(), tracks))
    }
}