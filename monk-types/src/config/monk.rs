use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::download::DownloadConfig;
use super::index::IndexConfig;
use super::store::StoreConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonkConfig {
    pub log: bool,
    pub data_dir: PathBuf,
    pub store: StoreConfig,
    pub index: IndexConfig,
    pub download: DownloadConfig,
}

impl Default for MonkConfig {
    fn default() -> Self {
        Self {
            log: false,
            data_dir: "data".into(),
            store: StoreConfig {
                path: "monk.sqlite".into(),
            },
            index: IndexConfig {
                path: "index".into(),
                index_on_add: true,
                summarize_on_add: true,
            },
            download: DownloadConfig {
                path: "downloads".into(),
                download_on_add: true,
            },
        }
    }
}

impl MonkConfig {
    pub fn store_path(&self) -> PathBuf {
        self.data_dir.join(&self.store.path)
    }

    pub fn index_path(&self) -> PathBuf {
        self.data_dir.join(&self.index.path)
    }

    pub fn download_path(&self) -> PathBuf {
        self.data_dir.join(&self.download.path)
    }
}
