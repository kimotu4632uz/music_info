use std::path::Path;
use taglib_picture::File;

use crate::fileio::picture::Picture;
use crate::traits::PictureFileIO;

pub struct TagLibPicture {
    file: File,
}

impl TagLibPicture {
    pub fn new<P: AsRef<Path>>(file: P) -> anyhow::Result<TagLibPicture> {
        let taglib_file = File::new(file)?;
        Ok(TagLibPicture{file: taglib_file})
    }
}

impl PictureFileIO for TagLibPicture {
    fn read(&self) -> anyhow::Result<Picture> {
        let (data, mime) = self.file.read_cover()?;
        Ok(Picture::new(data, mime))
    }

    fn write(&self, picture: &Picture) -> anyhow::Result<()> {
        self.file.write_cover(&picture.raw, picture.mime.as_str())?;
        Ok(())
    }
}
