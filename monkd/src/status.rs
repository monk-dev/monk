use std::fs::DirEntry;

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::index::Index;
use crate::metadata::offline_store::{OfflineStore, Status as OfflineStatus};
use crate::metadata::{file_store::FileStore, meta::IndexStatus, Meta};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaStatus {
    pub id: String,
    pub bytes_on_disk: usize,
    pub index_status: Option<IndexStatus>,
    pub offline_status: Option<OfflineStatus>,
}

impl MetaStatus {
    pub fn new(meta: &Meta, offline_store: &OfflineStore) -> Result<Self, Error> {
        let bytes_on_disk = calc_meta_bytes(&meta)?;
        let index_status = meta.index_status;
        let offline_status = offline_store
            .get(meta.id())
            .ok()
            .map(|d| &d.status)
            .cloned();

        tracing::info!("finished getting status");

        Ok(MetaStatus {
            id: meta.id().to_string(),
            bytes_on_disk,
            index_status,
            offline_status,
        })
    }
}

fn calc_meta_bytes(meta: &Meta) -> Result<usize, Error> {
    let data = serde_json::to_string(&meta)?;

    Ok(data.len())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileStoreStatus {
    pub item_count: usize,
    pub bytes_on_disk: usize,
    pub version: String,
}

impl FileStoreStatus {
    pub fn new(store: &FileStore) -> Result<Self, Error> {
        use std::fs::File;

        let file = File::open(store.file())?;
        let metadata = file.metadata()?;

        Ok(Self {
            item_count: store.data().len(),
            bytes_on_disk: metadata.len() as usize,
            version: store.version().to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfflineStoreStatus {
    pub item_count: usize,
    pub bytes_on_disk: usize,
}

impl OfflineStoreStatus {
    pub fn new(offline_store: &OfflineStore) -> Result<Self, Error> {
        use std::fs::read_dir;

        let item_count = offline_store.data().len();
        let mut bytes_on_disk = 0;

        for entry in read_dir(offline_store.file())? {
            match entry {
                // This should really be the job of the adapter
                Ok(entry) => {
                    bytes_on_disk += get_size(entry);
                }
                Err(_) => (),
            }
        }

        Ok(Self {
            item_count,
            bytes_on_disk,
        })
    }
}

// Recursively gets size of a file
fn get_size(entry: DirEntry) -> usize {
    use std::fs::read_dir;
    let mut size = 0;
    if entry.file_type().unwrap().is_dir() {
        for f in read_dir(entry.path()).unwrap() {
            size += get_size(f.unwrap());
        }
    } else {
        let metadata = entry.metadata().unwrap();
        size += metadata.len() as usize;
    }

    size
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TIndexStatus {
    pub item_count: usize,
    pub bytes_on_disk: usize,
}

impl TIndexStatus {
    pub fn new(index: &Index) -> Result<Self, Error> {
        use std::fs::read_dir;

        let item_count = index.count_indexed_items()?;
        let mut bytes_on_disk = 0;

        for entry in read_dir(index.folder())? {
            let metadata = entry?.metadata()?;
            bytes_on_disk += metadata.len() as usize;
        }

        Ok(Self {
            item_count,
            bytes_on_disk,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusResponse {
    pub meta: Option<MetaStatus>,
    pub file_store: Option<FileStoreStatus>,
    pub offline_store: Option<OfflineStoreStatus>,
    pub index_status: Option<TIndexStatus>,
}
