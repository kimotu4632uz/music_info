use std::path::{Path, PathBuf};

use crate::{info_struct::Metadata, traits::MetaFileIO};

pub struct Json {
    path: PathBuf,
}

impl Json {
    pub fn new<P: AsRef<Path>>(path: P) -> Json {
        return Json {
            path: path.as_ref().into(),
        };
    }

    pub fn to_string(meta: &Metadata) -> anyhow::Result<String> {
        let result = serde_json::to_string_pretty(meta)?;
        Ok(result)
    }
}

impl MetaFileIO for Json {
    fn read(&self) -> anyhow::Result<Metadata> {
        let json_str = std::fs::read_to_string(&self.path)?;
        let result = serde_json::from_str(&json_str)?;
        Ok(result)
    }

    fn write(&self, meta: &Metadata) -> anyhow::Result<()> {
        let json_str = serde_json::to_string_pretty(meta)?;
        std::fs::write(&self.path, json_str)?;
        Ok(())
    }
}
