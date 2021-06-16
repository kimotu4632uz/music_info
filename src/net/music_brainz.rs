use std::str::FromStr;
use std::io::Read;

use crate::traits::{FetchMeta, FetchPicture};
use crate::info_struct::*;
use crate::fileio::picture::Picture;

mod inner_structs;

pub struct MusicBrainz {
    client: crate::net::Client,
}

impl MusicBrainz {
    pub fn new() -> MusicBrainz {
        let client = crate::net::http_client();

        MusicBrainz { client: client }
    }

    fn get_mb(&self, url: &str, query: &[(&str, &str)], count: i32) -> anyhow::Result<serde_json::Value> {
        let mut client = self.client
            .get(url)
            .query("fmt", "json");
        
        for (key, val) in query {
            client = client.query(key, val);
        }

        let resp = client.call();

        match resp {
            Ok(r) => {
                let json: serde_json::Value = r.into_json()?;
                if let Some(err) = json.get("error") {
                    Err(anyhow::anyhow!("Failed to query for musicbrainz: {}", err))
                } else {
                    Ok(json)
                }
            },
            Err(ureq::Error::Status(c, _)) => {
                if c == 503 {
                    if count == 5 {
                        anyhow::bail!("Error: http error: 503 resp")
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    self.get_mb(url, query, count + 1)
                } else {
                    anyhow::bail!("Http error: {}", c)
                }
            },
            Err(ureq::Error::Transport(_)) => {
                anyhow::bail!("Http transport error")
            }
        }
    }
}

impl FetchMeta for MusicBrainz {
    fn query(&self, query: &str) -> anyhow::Result<Vec<(Metadata, AddInfo)>> {
        let json = self.get_mb("http://musicbrainz.org/ws/2/release/", &[("query", query)], 0)?;

        let release_ids = json["releases"].as_array().unwrap().into_iter().map(|e| e["id"].as_str().unwrap());
        let mut result = Vec::with_capacity(release_ids.len());

        for release in release_ids {
            let meta = self.fetch_all(release)?;
            result.push(meta);
        }

        Ok(result)
    }

    fn fetch_all(&self, id: &str) -> anyhow::Result<(Metadata, AddInfo)> {
        let release_json = self.get_mb(&format!("http://musicbrainz.org/ws/2/release/{}", id), &[("inc", "recordings+genres")], 0)?;
        let recording_ids = release_json["media"][0]["tracks"].as_array().unwrap().into_iter().map(|e| e["recording"]["id"].as_str().unwrap());

        let mut tracks = Vec::with_capacity(recording_ids.len());
        for recording_id in recording_ids {
            let recording_json = self.get_mb(&format!("http://musicbrainz.org/ws/2/recording/{}", recording_id), &[("inc", "artists")], 0)?;
            let recording: inner_structs::Recording = serde_json::from_value(recording_json)?;

            let artist = recording.artist_credit.iter().fold(String::new(), |acc, e| acc + &e.name + &e.joinphrase);

            tracks.push(Track::new(recording.title, artist));
        }

        let release: inner_structs::Release = serde_json::from_value(release_json)?;
        let date = release.date.split("-").next().unwrap();
        let date = u32::from_str(date)?;

        let mut add_info = Vec::new();

        if let Some(barcode) = release.barcode {
            add_info.push(("barcode".into(), barcode));
        }
        let cover_n = release.cover_art_archive.count;
        if cover_n > 0 {
            let mut cover_str = format!("count: {}, type: ", cover_n);
            if release.cover_art_archive.front { cover_str += "front, " }
            if release.cover_art_archive.back { cover_str += "back, " }
            if release.cover_art_archive.artwork { cover_str += "artwork" }
            add_info.push(("cover art".into(), cover_str))
        }
 
        Ok((
            Metadata::new(
                Some(id.to_string()),
                release.title,
                date,
                release.genres.first().map(|e| e.to_string()).unwrap_or_default(),
                tracks
            ),
            add_info
        ))
    }
}

impl FetchPicture for MusicBrainz {
    fn fetch_picture(&self, id: &str) -> anyhow::Result<Picture> {
        let pictures: inner_structs::Cover = self.client.get(&format!("http://coverartarchive.org/release/{}", id)).call()?.into_json()?;
        let front = pictures.images.iter().find(|e| e.front).unwrap_or(&pictures.images[0]);

        let img_resp = self.client.get(front.image.as_str()).call()?;
        let mime = img_resp.content_type().to_string();

        let data_len = img_resp.header("Content-Length")
            .and_then(|s| s.parse::<usize>().ok())
            .expect("Error: image responce not include content-length");

        let mut data = Vec::with_capacity(data_len);
        img_resp.into_reader().read_to_end(&mut data)?;

        Ok(Picture::new(data, mime))
    }
}