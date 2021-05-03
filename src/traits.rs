use crate::info_struct::{Metadata, AddInfo};
use crate::fileio::picture::Picture;

pub trait MetaFileIO {
    fn read(&self) -> anyhow::Result<Metadata>;
    fn write(&self, meta: &Metadata) -> anyhow::Result<()>;
}

pub trait PictureFileIO {
    fn read(&self) -> anyhow::Result<Picture>;
    fn write(&self, picture: &Picture) -> anyhow::Result<()>;
}

pub trait FetchMeta {
    fn query(&self, query: &str) -> anyhow::Result<Vec<(Metadata, Vec<(String, String)>)>>;
    fn fetch_all(&self, id: &str) -> anyhow::Result<(Metadata, AddInfo)>;

    fn fetch(&self, id: &str) -> anyhow::Result<Metadata> {
        self.fetch_all(id).map(|x| x.0)
    }
}

pub trait FetchPicture {
    fn fetch_picture(&self, id: &str) -> anyhow::Result<Picture>;
}