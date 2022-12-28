use std::{io::Read, path::Path, str::FromStr};

use crate::{
    fileio::picture::Picture,
    info_struct::*,
    net::oauth2::*,
    traits::{FetchMeta, FetchPicture},
};

mod inner_structs;

pub struct Spotify {
    client: Client,
}

impl Spotify {
    pub fn new<P: AsRef<Path>>(cred: P, token: P) -> anyhow::Result<Spotify> {
        let client = Client::new(
            cred,
            token,
            "https://accounts.spotify.com/api/token".to_string(),
        )?;
        Ok(Spotify { client })
    }
}

impl FetchMeta for Spotify {
    fn query(&self, query: &str) -> anyhow::Result<Vec<(Metadata, AddInfo)>> {
        let resp: serde_json::Value = self
            .client
            .get("https://api.spotify.com/v1/search")?
            .query("q", query)
            .query("type", "album")
            .query("market", "JP")
            .call()?
            .into_json()?;

        if resp["albums"]["total"].as_i64().unwrap_or(0) > 0 {
            let ids = resp["albums"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v["id"].as_str().unwrap().to_string());
            let mut result = Vec::with_capacity(ids.len());

            for id in ids {
                let meta = self.fetch_all(&id)?;
                result.push(meta);
            }

            Ok(result)
        } else {
            Ok(vec![])
        }
    }

    fn fetch_all(&self, id: &str) -> anyhow::Result<(Metadata, AddInfo)> {
        let mut resp: inner_structs::Album = self
            .client
            .get(&format!("https://api.spotify.com/v1/albums/{}", id))?
            .query("market", "JP")
            .call()?
            .into_json()?;

        let album = resp.name;
        let date = match resp.release_date_precision.as_str() {
            "year" => u32::from_str(&resp.release_date)?,
            "month" | "day" => {
                let date = resp.release_date.split("-").next().unwrap();
                u32::from_str(date)?
            }
            _ => 0,
        };
        let genre = resp.genres.join(", ");
        let mut tracks = Vec::with_capacity(resp.tracks.items.len());

        let tracks_from = resp.tracks.items.as_mut_slice();
        tracks_from.sort_by_key(|i| i.track_number);

        for track in tracks_from {
            let title = track.name.clone();
            let artist = track
                .artists
                .iter()
                .map(|x| x.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            tracks.push(Track::new(title, artist));
        }

        let ext_ids = resp
            .external_ids
            .iter()
            .map(|(k, v)| (k.clone(), format!("{}", v)));

        let add_info = if !resp.images.is_empty() {
            let image = resp
                .images
                .iter()
                .map(|x| format!("{} x {}", x.height, x.width))
                .collect::<Vec<_>>()
                .join(", ");

            vec![("image".to_string(), image)]
                .into_iter()
                .chain(ext_ids)
                .collect()
        } else {
            ext_ids.collect()
        };

        Ok((
            Metadata::new(Some(id.to_string()), album, date, genre, tracks),
            add_info,
        ))
    }
}

impl FetchPicture for Spotify {
    fn fetch_picture(&self, id: &str) -> anyhow::Result<Picture> {
        let mut resp: inner_structs::Album = self
            .client
            .get(&format!("https://api.spotify.com/v1/albums/{}", id))?
            .query("market", "JP")
            .call()?
            .into_json()?;

        resp.images.sort_by_key(|x| x.height);
        let url = resp
            .images
            .last()
            .ok_or(anyhow::anyhow!("no cover art found"))?
            .url
            .as_str();

        let img_resp = crate::net::http_client().get(url).call()?;

        let mime = img_resp.content_type().to_string();
        let data_len = img_resp
            .header("Content-Length")
            .and_then(|s| s.parse::<usize>().ok())
            .expect("Error: image responce not include content-length");

        let mut data = Vec::with_capacity(data_len);
        img_resp.into_reader().read_to_end(&mut data)?;

        Ok(Picture::new(data, mime))
    }
}
