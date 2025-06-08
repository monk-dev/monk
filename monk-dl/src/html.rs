use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use monk_types::{HtmlDownloader, Item};
use monolith::opts::Options;

pub const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:96.0) Gecko/20100101 Firefox/96.0";

pub struct MonolithDownloader {
    pub folder: PathBuf,
}

impl MonolithDownloader {
    pub fn new(folder: PathBuf) -> Self {
        Self { folder }
    }
}

#[async_trait::async_trait]
impl HtmlDownloader for MonolithDownloader {
    async fn download_html(&self, item: &Item) -> anyhow::Result<PathBuf> {
        let filename = format!("{}.html", item.id);
        let file_path = self.folder.join(filename);

        if let Some(url) = item.url.clone() {
            tracing::info!("downloading html in monolith");

            let opts = Options {
                base_url: Some(url.clone()),
                charset: Some("UTF-8".to_string()),
                isolate: true,
                silent: true,
                timeout: 120,
                target: file_path.display().to_string(),
                ..Default::default()
            };

            let url = url.parse()?;

            let file_path = tokio::task::spawn_blocking(move || {
                let mut cache = HashMap::new();
                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(60))
                    .danger_accept_invalid_certs(false)
                    .user_agent(DEFAULT_USER_AGENT)
                    .build()
                    .unwrap();

                tracing::info!(%url, "retrieving asset");
                let (data, _final_url, _media_type, char_set) =
                    monolith::utils::retrieve_asset(&mut cache, &client, &url, &url, &opts, 1)
                        .context("unable to retrieve asset")?;

                tracing::info!("converting html to dom");
                let dom = monolith::html::html_to_dom(&data, char_set.clone());

                // tracing::info!("[{}] Embedding asset: {}", item.id, url.as_str());

                tracing::info!("walking and embedding assets");
                monolith::html::walk_and_embed_assets(
                    &mut cache,
                    &client,
                    &url,
                    &dom.document,
                    &opts,
                    1,
                );

                tracing::info!("serializing document");
                let html: Vec<u8> = monolith::html::serialize_document(dom, char_set, &opts);

                // tracing::info!("[{}] document file_path: {}", item.id, file_path.display());
                // tracing::info!("Writing html file: {} => {}", item.id, file_path.display());

                std::fs::write(&file_path, html)?;

                // tracing::info!("Successfully extracted asset: {}", item.id);

                Ok::<_, anyhow::Error>(file_path)
            })
            .await??;

            Ok(file_path)
        } else {
            anyhow::bail!("item is missing a url");
        }
    }
}
