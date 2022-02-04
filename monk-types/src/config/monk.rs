use serde::{Deserialize, Serialize};

use super::download::DownloadConfig;
use super::index::IndexConfig;
use super::store::StoreConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonkConfig {
    pub log: bool,
    pub store: StoreConfig,
    pub index: IndexConfig,
    pub download: DownloadConfig,
}

impl Default for MonkConfig {
    fn default() -> Self {
        Self {
            log: true,
            store: StoreConfig {
                path: "sqlite:monk.sqlite".into(),
            },
            index: IndexConfig {
                path: "index".into(),
                index_on_add: true,
            },
            download: DownloadConfig {
                path: "downloads".into(),
                download_on_add: true,
            },
        }
    }
}
