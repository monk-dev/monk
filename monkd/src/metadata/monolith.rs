use monolith::html::{html_to_dom, stringify_document, walk_and_embed_assets};
use monolith::utils::retrieve_asset;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::metadata::Meta;

/// From monolith/src/args.rs
const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0";

#[tracing::instrument(skip(store))]
pub fn download_meta(meta: &Meta, store: impl AsRef<Path>) -> Result<PathBuf, Error> {
    if let Some(url) = meta.url() {
        let mut cache = HashMap::new();
        let mut header_map = HeaderMap::new();
        header_map.insert(
            USER_AGENT,
            HeaderValue::from_str(DEFAULT_USER_AGENT).unwrap(),
        );

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .danger_accept_invalid_certs(false)
            .default_headers(header_map)
            .build()
            .unwrap();

        tracing::info!("Retrieving asset: {} => {}", meta.id(), url.as_str());

        let (data, _final_url, _media_type) =
            retrieve_asset(&mut cache, &client, url.as_str(), url.as_str(), true)?;

        let dom = html_to_dom(&String::from_utf8(data)?);

        tracing::info!("Embedding asset: {} => {}", meta.id(), url.as_str());

        walk_and_embed_assets(
            &mut cache,
            &client,
            url.as_str(),
            &dom.document,
            false,
            false,
            false,
            false,
            false,
            true,
        );

        let html: String = stringify_document(&dom.document, false, false, false, false, true);
        let filename = format!("{}.html", meta.id());
        let file_path = store.as_ref().join(filename);

        tracing::info!(
            "Writing html file: {} => {}",
            meta.id(),
            file_path.display()
        );

        fs::write(&file_path, html)?;

        tracing::info!("Successfully extracted asset: {}", meta.id());

        Ok(file_path)
    } else {
        tracing::info!("Meta has no url: {}", meta.id());

        Err(Error::NoUrl(meta.id().to_string()))
    }
}
