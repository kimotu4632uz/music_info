use std::{borrow::Borrow, path::Path};

use taglib::{File, FileError};

use crate::{info_struct::*, traits::MetaFileIO};

pub struct TagLib {
    files: Vec<Option<File>>,
}

impl TagLib {
    pub fn new<P, I>(paths: I) -> anyhow::Result<TagLib>
    where
        P: AsRef<Path>,
        I: IntoIterator,
        I::Item: Borrow<Option<P>>,
    {
        let result = paths
            .into_iter()
            .map(|b| {
                b.borrow().as_ref().and_then(|p| {
                    File::new(p)
                        .map_err(|e| {
                            match e {
                                FileError::InvalidFileName => {
                                    println!("Warning: invalid file name.")
                                }
                                FileError::InvalidFile => {
                                    println!("Warning: invalid file structure")
                                }
                                _ => println!(""),
                            };
                            e
                        })
                        .ok()
                })
            })
            .collect();

        Ok(TagLib { files: result })
    }
}

impl MetaFileIO for TagLib {
    fn read(&self) -> anyhow::Result<Metadata> {
        let first = self.files.iter().find_map(Option::as_ref).unwrap();
        let first_tag = first
            .tag()
            .map_err(|_| anyhow::anyhow!("Error: no available tag found."))?;
        let mut tracks = Vec::with_capacity(self.files.len());

        for file in &self.files {
            if let Some(file) = file {
                let tag = file
                    .tag()
                    .map_err(|_| anyhow::anyhow!("Error: no available tag found."))?;
                tracks.push(Track::new(
                    tag.title().unwrap_or_default(),
                    tag.artist().unwrap_or_default(),
                ));
            } else {
                tracks.push(Track::new("", ""));
            }
        }

        Ok(Metadata::new(
            None,
            first_tag.album().unwrap_or_default(),
            first_tag.year().unwrap_or_default(),
            first_tag.genre().unwrap_or_default(),
            tracks,
        ))
    }

    fn write(&self, meta: &Metadata) -> anyhow::Result<()> {
        for (i, file) in self.files.iter().enumerate() {
            if let Some(file) = file {
                let mut tag = file.tag().unwrap();
                tag.set_track((i + 1) as u32);
                tag.set_album(&meta.album);
                tag.set_year(meta.date);
                tag.set_genre(&meta.genre);

                tag.set_title(&meta.tracks[i].title);
                tag.set_artist(&meta.tracks[i].artist);
                let result = file.save();
                if !result {
                    anyhow::bail!("Error: could not write metadata")
                }
            }
        }

        Ok(())
    }
}
