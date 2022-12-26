use clap::{Parser, Subcommand};

use std::path::PathBuf;
use std::str::FromStr;

use music_info::{
    fileio::{Json, Picture, TagLib, TagLibPicture},
    info_struct::{Metadata, Track},
    net::{MusicBrainz, Spotify},
    traits::*,
};

#[derive(Parser, Debug)]
#[clap(author, about, version)]
struct Cmd {
    #[clap(subcommand)]
    op: Opr,
}

#[derive(Subcommand, Debug)]
enum Opr {
    Read {
        /// output json file
        #[clap(short, long)]
        json: Option<PathBuf>,

        /// output picture file
        #[clap(short, long)]
        picture: Option<PathBuf>,

        /// source audio file
        #[clap(required = true)]
        audio: Vec<String>,
    },
    Write {
        /// input json file
        #[clap(short, long)]
        json: PathBuf,

        /// input picture file
        #[clap(short, long)]
        picture: Option<PathBuf>,

        /// target audio files
        #[clap(required = true)]
        audio: Vec<String>,
    },
    Query {
        #[clap(subcommand)]
        opr: QueryOpr,
    },
    Fetch {
        #[clap(subcommand)]
        opr: FetchOpr,
    },
    Template {
        #[clap(subcommand)]
        opr: TempOpr,
    },
}

#[derive(Subcommand, Debug)]
enum QueryOpr {
    MusicBrainz {
        /// query parameter to find info, syntax: https://musicbrainz.org/doc/Indexed_Search_Syntax
        query: String,
    },
    Spotify {
        /// query parameter to find info
        query: String,
    },
}

#[derive(Subcommand, Debug)]
enum FetchOpr {
    MusicBrainz {
        /// output file to save json
        #[clap(short, long)]
        output: Option<PathBuf>,

        /// if present, save picture to PICTURE
        #[clap(short, long)]
        picture: Option<PathBuf>,

        /// MBID of target release
        id: String,
    },
    Spotify {
        /// output file to save json
        #[clap(short, long)]
        output: Option<PathBuf>,

        /// if present, save picture to PICTURE
        #[clap(short, long)]
        picture: Option<PathBuf>,

        /// spotify ID of target album
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum TempOpr {
    Json {
        /// output file to write template
        #[clap(short, long)]
        output: Option<PathBuf>,
    },
}

fn audio_files_parser(files: Vec<String>) -> anyhow::Result<Vec<Option<String>>> {
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
    let arg = Cmd::parse();

    match arg.op {
        Opr::Read {
            json,
            picture,
            audio,
        } => {
            let files = audio_files_parser(audio)?;
            let first_file = files.iter().find_map(|e| e.as_ref()).unwrap().to_owned();

            let result = TagLib::new(files)?.read()?;

            if let Some(path) = json {
                Json::new(path).write(&result)?;
            } else {
                print!("{}", Json::to_string(&result)?);
            }

            if let Some(path) = picture {
                TagLibPicture::new(first_file)?.read()?.write(path)?;
            }
        }
        Opr::Write {
            json,
            picture,
            audio,
        } => {
            let files = audio_files_parser(audio)?;

            let meta = Json::new(json).read()?;

            if let Some(path) = picture {
                let pic = Picture::read(path)?;

                for file in files.iter().filter_map(|e| e.as_ref()) {
                    TagLibPicture::new(file)?.write(&pic)?;
                }
            }

            TagLib::new(files)?.write(&meta)?;
        }
        Opr::Query { opr } => {
            let result = match opr {
                QueryOpr::MusicBrainz { query } => MusicBrainz::new().query(&query)?,
                QueryOpr::Spotify { query } => {
                    let default_cred_path = dirs::home_dir().unwrap().join(".spotify_cred.json");
                    let default_token_path = dirs::home_dir().unwrap().join(".spotify_token.json");

                    Spotify::new(default_cred_path, default_token_path)?.query(&query)?
                }
            };

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
        }
        Opr::Fetch { opr } => {
            let (result, output, picture) = match opr {
                FetchOpr::MusicBrainz {
                    output,
                    picture,
                    id,
                } => {
                    let client = MusicBrainz::new();
                    let result = client.fetch(&id)?;

                    let picture = picture
                        .map(|path| client.fetch_picture(&id).map(|data| (data, path)))
                        .transpose()?;

                    (result, output, picture)
                }
                FetchOpr::Spotify {
                    output,
                    picture,
                    id,
                } => {
                    let default_cred_path = dirs::home_dir().unwrap().join(".spotify_cred.json");
                    let default_token_path = dirs::home_dir().unwrap().join(".spotify_token.json");

                    let client = Spotify::new(default_cred_path, default_token_path)?;
                    let result = client.fetch(&id)?;

                    let picture = picture
                        .map(|path| client.fetch_picture(&id).map(|data| (data, path)))
                        .transpose()?;

                    (result, output, picture)
                }
            };

            if let Some(out) = output {
                Json::new(out).write(&result)?;
            } else {
                print!("{}", Json::to_string(&result)?);
            }

            if let Some((data, path)) = picture {
                data.write(path)?;
            }
        }
        Opr::Template { opr } => {
            let mut default = Metadata::default();
            default.tracks = vec![Track::default()];

            match opr {
                TempOpr::Json { output } => {
                    if let Some(path) = output {
                        Json::new(path).write(&default)?;
                    } else {
                        print!("{}", Json::to_string(&default)?);
                    }
                }
            }
        }
    };

    Ok(())
}
