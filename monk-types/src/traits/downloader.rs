use std::path::PathBuf;

use crate::{Blob, Item, Store};

/// Downloads and persists web content for offline access and processing.
/// 
/// The Downloader trait handles fetching content from URLs associated with items
/// and storing them locally as blobs. It manages the complete download lifecycle
/// including content fetching, local storage, and metadata creation.
/// 
/// # Workflow
/// 1. Takes an item with a URL
/// 2. Downloads the content from the URL
/// 3. Stores the content locally
/// 4. Creates a blob entry in the store with metadata
/// 5. Returns the blob for further processing
#[async_trait::async_trait]
pub trait Downloader {
    /// Downloads content from an item's URL and stores it as a blob.
    /// 
    /// Takes an item containing a URL and downloads its content to local storage.
    /// Creates a blob entry in the store with metadata including file path, hash,
    /// and content type. The blob can then be used for content extraction and indexing.
    async fn download(
        &self,
        store: &(dyn Store + Send + Sync + 'static),
        item: &Item,
    ) -> anyhow::Result<Blob>;
}

/// Downloads HTML content from web pages.
#[async_trait::async_trait]
pub trait HtmlDownloader: Send + Sync + 'static {
    /// Downloads HTML content and returns the local file path.
    async fn download_html(&self, item: &Item) -> anyhow::Result<PathBuf>;
}
