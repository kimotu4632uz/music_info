use std::path::Path;

pub struct Picture {
    pub raw: Vec<u8>,
    pub mime: String,
}

impl Picture {
    pub fn new(raw: Vec<u8>, mime: String) -> Picture {
        Picture{ raw: raw, mime: mime}
    }

    pub fn read<P: AsRef<Path>>(file: P) -> anyhow::Result<Picture> {
        let result = std::fs::read(file.as_ref())?;
        let mime = mime_guess::from_path(file).first().map(|m| m.as_ref().to_string()).unwrap_or_default();
        Ok(Picture::new(result, mime))
    }

    pub fn write<P: AsRef<Path>>(&self, file: P) -> anyhow::Result<()> {
        let ext = mime_guess::get_mime_extensions_str(&self.mime).unwrap_or_default()[0];
        let path = file.as_ref().with_extension(ext);

        std::fs::write(path, &self.raw)?;
        Ok(())
    }
}