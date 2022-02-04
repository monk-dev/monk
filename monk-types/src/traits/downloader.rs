use std::path::PathBuf;

use crate::{Blob, Item, Store};

#[async_trait::async_trait]
pub trait Downloader {
    async fn download(
        &self,
        store: &(dyn Store + Send + Sync + 'static),
        item: &Item,
    ) -> anyhow::Result<Blob>;
}

#[async_trait::async_trait]
pub trait HtmlDownloader: Send + Sync + 'static {
    async fn download_html(&self, item: &Item) -> anyhow::Result<PathBuf>;
}
