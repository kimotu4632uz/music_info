use std::path::PathBuf;
use crate::info_struct::Metadata;
use crate::traits::FileIO;

struct Json {
    path: PathBuf,
}

impl Json {
    pub fn new(path: PathBuf) -> Json {
        return Json{ path: path };
    }
}

impl FileIO for Json {
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