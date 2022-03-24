use std::path::{Path, PathBuf};

use anyhow::bail;
use monk_types::config::DownloadConfig;
use monk_types::{Blob, Downloader, HtmlDownloader, Item, Store};
use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tracing::info;

use crate::html::{MonolithDownloader, DEFAULT_USER_AGENT};

pub struct MonkDownloader {
    _config: DownloadConfig,
    data_dir: PathBuf,
    html: Box<dyn HtmlDownloader + Send + Sync>,
}

impl MonkDownloader {
    pub async fn from_config(
        data_dir: impl AsRef<Path>,
        config: &DownloadConfig,
    ) -> anyhow::Result<Self> {
        let path = data_dir.as_ref().join(&config.path);
        tokio::fs::create_dir_all(&path).await?;

        let html = Box::new(MonolithDownloader::new(path.clone()));

        Ok(Self {
            _config: config.clone(),
            data_dir: path,
            html,
        })
    }

    async fn download_get(&self, item: &Item) -> anyhow::Result<PathBuf> {
        let url = if let Some(path) = item.url.as_deref() {
            path
        } else {
            bail!("item must have a path");
        };

        let path = self.data_dir.join(item.id.to_string());
        let file = File::create(&path).await?;
        let mut writer = BufWriter::new(file);

        let client = reqwest::Client::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .build()?;

        let mut resp = client.get(url).send().await?;

        info!(content_length = ?resp.content_length(), "writing bytes");
        while let Some(chunk) = resp.chunk().await? {
            writer.write_all(&chunk).await?;
        }

        Ok(path)
    }
}

#[async_trait::async_trait]
impl Downloader for MonkDownloader {
    async fn download(
        &self,
        store: &(dyn Store + Send + Sync + 'static),
        item: &Item,
    ) -> anyhow::Result<Blob> {
        tracing::info!("beginning download");

        let url = if let Some(url) = item.url.as_deref() {
            url
        } else {
            anyhow::bail!("item is missing a url");
        };

        let mut managed = true;
        let mut mime_type = mime_guess::from_path(url).first_or(mime::TEXT_HTML_UTF_8);

        let path = if let Ok(_) = tokio::fs::metadata(url).await {
            // This is a local path, instruct the store not to delete it if the blob is deleted:
            managed = false;

            PathBuf::from(url)
        } else if url.starts_with("http") {
            tracing::info!(%mime_type, "guessed mime type");

            if mime_type == mime::TEXT_HTML_UTF_8 {
                // If it's specifically html, download with monolith
                match self.html.download_html(&item).await {
                    Ok(path) => path,
                    Err(error) => {
                        info!(%error, "error downloading as html, falling back to `get`");
                        self.download_get(&item).await?.canonicalize()?
                    }
                }
            } else {
                // Download with a normal GET
                self.download_get(&item).await?.canonicalize()?
            }
        } else {
            anyhow::bail!("unsupported file schema")
        };

        info!("inferring mime type from file");
        let infer_path = path.clone();

        let type_opt =
            tokio::task::spawn_blocking(move || infer::get_from_path(&infer_path)).await??;

        if let Some(ty) = type_opt {
            mime_type = ty.mime_type().parse()?;
        }

        let hash = calculate_hash(&path).await?;

        info!(%mime_type, "file mime type");
        info!(%hash, "file hash");

        store
            .add_blob(
                item.id.clone(),
                url.to_string(),
                hash,
                mime_type.to_string(),
                path.display().to_string(),
                managed,
            )
            .await
    }
}

async fn calculate_hash(path: impl AsRef<Path>) -> anyhow::Result<String> {
    let file = tokio::fs::File::open(path).await?;
    let mut reader = BufReader::new(file);

    let mut hasher = Sha256::new();
    let mut buffer = Vec::with_capacity(4096);
    loop {
        let count = reader.read(&mut buffer).await?;
        if count == 0 {
            break;
        }

        hasher.update(&buffer[0..count]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
