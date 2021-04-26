use crate::info_struct::Metadata;

pub trait FileIO {
    fn read(&self) -> anyhow::Result<Metadata>;
    fn write(&self, meta: &Metadata) -> anyhow::Result<()>;
}

pub trait FetchMeta {
    fn query(&self, query: &str) -> anyhow::Result<Vec<Metadata>>;
    fn fetch(&self, id: &str) -> anyhow::Result<Metadata>;
}