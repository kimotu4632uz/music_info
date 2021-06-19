use clap::clap_app;
use std::str::FromStr;

use music_info::info_struct::{Metadata, Track};
use music_info::traits::*;

fn audio_files_parser(files: clap::Values) -> anyhow::Result<Vec<Option<String>>> {
    let mut result = Vec::new();
    let mut idx = 0;

    for file in files {
        let ext: Vec<_> = file.rsplitn(2, ":").collect();
        if ext.len() == 1 {
            result.push(Some(file.into()));
            idx += 1;
        } else {
            idx = usize::from_str(ext[0])? - 1;
            if result.len() < idx {
                result.resize(idx, None);
            }
            result.push(Some(ext[1].into()));
        }
    }

    Ok(result)
}

fn main() -> anyhow::Result<()> {
//            (@arg json: index(2) -j --json [JSON] "output json file")
    let mut app = clap_app!(music_info =>
        (version: "0.3.0")
        (@subcommand read =>
            (@arg json: -j --json [JSON] "output json file")
            (@arg picture: -p --picture  [PICTURE] "picture file path to write")
            (@arg audios: <AUDIOS>... "source audio files")
        )
        (@subcommand write =>
            (@arg json: -j --json <JSON> "read from json file")
            (@arg picture: -p --picture [PICTURE] "picture file path to write")
            (@arg audios: <AUDIOS>... "target audio files")
        )
        (@subcommand query =>
            (@subcommand music_brainz =>
                (@arg query: <QUERY> "query parameter to find info, syntax: https://musicbrainz.org/doc/Indexed_Search_Syntax")
            )
            (@subcommand spotify =>
                (@arg query: <QUERY> "query parameter to find info")
            )
        )
        (@subcommand fetch =>
            (@subcommand music_brainz =>
                (@arg id: <ID> "MBID of target release")
                (@arg output: -o --output [OUTPUT] "output file to save json")
                (@arg picture: -p --picture [PICTURE] "if present, save picture to PICTURE")
            )
            (@subcommand spotify =>
                (@arg id: <ID> "spotify ID of target album")
                (@arg output: -o --output [OUTPUT] "output file to save json")
                (@arg picture: -p --picture [PICTURE] "if present, save picture to PICTURE")
            )
        )
        (@subcommand template =>
            (@subcommand json =>
                (@arg output: -o --output [OUTPUT] "output file to write template")
            )
        )
    );

    if std::env::args().len() == 1 {
        app.print_help()?;
        return Ok(())
    }

    let matches = app.get_matches();

    if let Some(s_match) = matches.subcommand_matches("read") {
        let output = s_match.value_of("json");
        let picture = s_match.value_of("picture");
        let files = s_match.values_of("audios").unwrap();

        let files_fix = audio_files_parser(files)?;
        let first_file = files_fix.iter().find_map(|e| e.as_ref()).unwrap().to_owned();

        let taglib = music_info::fileio::taglib::TagLib::new(files_fix)?;
        let result = taglib.read()?;

        if let Some(json) = output {
            music_info::fileio::json::Json::new(json).write(&result)?;
        } else {
            let result = music_info::fileio::json::Json::to_string(&result)?;
            print!("{}", result);
        }

        if let Some(path) = picture {
            let taglib_pic = music_info::fileio::taglib_pic::TagLibPicture::new(first_file)?;
            let picture = taglib_pic.read()?;
            picture.write(path)?;
        }

    } else if let Some(s_match) = matches.subcommand_matches("write") {
        let json = s_match.value_of("json").unwrap();
        let picture = s_match.value_of("picture");
        let files = s_match.values_of("audios").unwrap();

        let files_fix = audio_files_parser(files)?;

        let json = music_info::fileio::json::Json::new(json);
        let meta = json.read()?;

        if let Some(path) = picture {
            let pic = music_info::fileio::picture::Picture::read(path)?;
            for file in files_fix.iter().filter_map(|e| e.as_ref()) {
                let taglib_pic = music_info::fileio::taglib_pic::TagLibPicture::new(file)?;
                taglib_pic.write(&pic)?;
            }
        }

        let taglib = music_info::fileio::taglib::TagLib::new(files_fix)?;
        taglib.write(&meta)?;

    } else if let Some(s_match) = matches.subcommand_matches("query") {
        let result = if let Some(ss_match) = s_match.subcommand_matches("music_brainz") {
            let query = ss_match.value_of("query").unwrap();

            let client = music_info::net::music_brainz::MusicBrainz::new();
            let result = client.query(query)?;
            Some(result)
        } else if let Some(ss_match) = s_match.subcommand_matches("spotify") {
            let query = ss_match.value_of("query").unwrap();

            let default_cred_path = dirs::home_dir().unwrap().join(".spotify_cred.json");
            let default_token_path = dirs::home_dir().unwrap().join(".spotify_token.json");

            let client = music_info::net::spotify::Spotify::new(default_cred_path, default_token_path)?;
            let result = client.query(query)?;
            Some(result)
        } else {
            None
        }.unwrap();

        let last_idx = result.len() - 1;
        println!("query result:");
        for (idx, (meta, add_info)) in result.iter().enumerate() {
            print!("{}", meta);

            if add_info.len() > 0 {
                println!("additional info:")
            }
            for (k, v) in add_info {
                println!("  {}: {}", k, v);
            }

            if idx != last_idx {
                print!("\n\n")
            }
        }

    } else if let Some(s_match) = matches.subcommand_matches("fetch") {
        let (result, output, picture) = if let Some(ss_match) = s_match.subcommand_matches("music_brainz") {
            let id = ss_match.value_of("id").unwrap();
            let output = ss_match.value_of("output");
            let picture = ss_match.value_of("picture");

            let client = music_info::net::music_brainz::MusicBrainz::new();
            let result = client.fetch(id)?;

            let picture = picture.map(|path| {
                client.fetch_picture(id).map(|data| (data, path))
            }).transpose()?;

            Some((result, output, picture))
        } else if let Some(ss_match) = s_match.subcommand_matches("spotify") {
            let id = ss_match.value_of("id").unwrap();
            let output = ss_match.value_of("output");
            let picture = ss_match.value_of("picture");

            let default_cred_path = dirs::home_dir().unwrap().join(".spotify_cred.json");
            let default_token_path = dirs::home_dir().unwrap().join(".spotify_token.json");

            let client = music_info::net::spotify::Spotify::new(default_cred_path, default_token_path)?;
            let result = client.fetch(id)?;

            let picture = picture.map(|path| {
                client.fetch_picture(id).map(|data| (data, path))
            }).transpose()?;

            Some((result, output, picture))
        } else {
            None
        }.unwrap();


        if let Some(out) = output {
            music_info::fileio::json::Json::new(out).write(&result)?;
        } else {
            let json = music_info::fileio::json::Json::to_string(&result)?;
            print!("{}", json);
        }

        if let Some((data, path)) = picture {
            data.write(path)?;
        }
    } else if let Some(s_match) = matches.subcommand_matches("template") {
        let mut default = Metadata::default();
        default.tracks = vec![Track::default()];

        if let Some(ss_match) = s_match.subcommand_matches("json") {
            let output = ss_match.value_of("output");
            if let Some(out) = output {
                music_info::fileio::json::Json::new(out).write(&default)?;
            } else {
                let json = music_info::fileio::json::Json::to_string(&default)?;
                print!("{}", json);
            }
        }
    }

    Ok(())
}
