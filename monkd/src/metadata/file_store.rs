// use std::path::Path;
// use tokio::fs::File;
// use tokio::io::{AsyncBufReadExt, BufReader};
// use tokio::stream::{Stream, StreamExt};

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, instrument};

use super::Meta;
use crate::error::Error;

pub const CURRENT_FILE_STORE_VERSION: &'static str = "0.0.0";

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FileStore {
    version: String,
    metadata: Vec<Meta>,
    #[serde(skip)]
    file: PathBuf,
    #[serde(skip)]
    dirty: bool,
}

impl FileStore {
    pub fn empty() -> Self {
        Self {
            version: CURRENT_FILE_STORE_VERSION.to_string(),
            metadata: Vec::new(),
            file: PathBuf::new(),
            dirty: false,
        }
    }

    pub fn push(&mut self, meta: Meta) -> Result<(), Error> {
        self.dirty = true;

        match self.metadata.binary_search_by_key(&meta.id(), |m| m.id()) {
            Ok(_) => Err(Error::AlreadyExists(meta.id().to_string())),
            Err(index) => {
                self.metadata.insert(index, meta);
                self.dirty = true;
                Ok(())
            }
        }
    }

    pub fn file(&self) -> &Path {
        &self.file
    }

    pub fn read_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        check_path(&path)?;

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        let mut store: FileStore = serde_json::from_reader(reader)?;
        store.file = path.as_ref().into();
        store.metadata.sort_by(|l, r| l.id().cmp(r.id()));

        Ok(store)
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, self)?;

        Ok(())
    }

    pub fn get(&self, id: impl AsRef<str>) -> Result<&Meta, Error> {
        let ids: Vec<usize> = self
            .metadata
            .iter()
            .enumerate()
            .filter(|(_, m)| m.id().starts_with(id.as_ref()))
            .map(|(i, _)| i)
            .collect();

        tracing::info!("Ids: {:?}", ids);

        if ids.len() > 1 {
            return Err(Error::TooManyMetas(
                id.as_ref().into(),
                ids.into_iter().map(|i| self.metadata[i].clone()).collect(),
            ));
        } else if ids.is_empty() {
            return Err(Error::IdNotFound(id.as_ref().into()));
        }

        // self.metadata.iter().find(|m| m.id() == id.as_ref())
        Ok(&self.metadata[ids[0]])
    }

    pub fn get_mut(&mut self, id: impl AsRef<str>) -> Result<&mut Meta, Error> {
        let ids: Vec<usize> = self
            .metadata
            .iter()
            .enumerate()
            .filter(|(_, m)| m.id().starts_with(id.as_ref()))
            .map(|(i, _)| i)
            .collect();

        tracing::info!("Ids: {:?}", ids);

        if ids.len() > 1 {
            return Err(Error::TooManyMetas(
                id.as_ref().into(),
                ids.into_iter().map(|i| self.metadata[i].clone()).collect(),
            ));
        } else if ids.is_empty() {
            return Err(Error::IdNotFound(id.as_ref().into()));
        }

        self.dirty = true;

        // self.metadata.iter().find(|m| m.id() == id.as_ref())
        Ok(&mut self.metadata[ids[0]])
    }

    pub fn index(&self, idx: usize) -> &Meta {
        &self.metadata[idx]
    }

    pub fn update(&mut self, id: impl AsRef<str>, data: Meta) -> Result<(), Error> {
        if id.as_ref() != data.id {
            return Err(Error::UnequalIds);
        }

        let meta = self.get_mut(&id)?;
        *meta = data;

        self.dirty = true;

        Ok(())
    }

    pub fn delete(&mut self, id: impl AsRef<str>) -> Result<Meta, Error> {
        let ids: Vec<usize> = self
            .metadata
            .iter()
            .enumerate()
            .filter(|(_, m)| m.id().starts_with(id.as_ref()))
            .map(|(i, _)| i)
            .collect();

        tracing::info!("Ids: {:?}", ids);

        if ids.len() > 1 {
            return Err(Error::TooManyIds(id.as_ref().into(), ids));
        } else if ids.is_empty() {
            return Err(Error::IdNotFound(id.as_ref().into()));
        }

        tracing::info!("Deleting: `{}`", id.as_ref());
        self.dirty = true;

        let removed = self.metadata.swap_remove(ids[0]);

        Ok(removed)
    }

    pub fn data(&self) -> &[Meta] {
        &self.metadata
    }

    #[instrument(skip(self))]
    pub fn commit(&mut self) -> Result<(), Error> {
        if self.dirty {
            info!("FileStore dirty: {}", self.file().display());

            self.write_file(self.file())?;
            self.dirty = false;
        } else {
            info!("FileStore clean: {}", self.file().display());
        }

        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub async fn commit_loop(
        handle: Arc<RwLock<FileStore>>,
        delay: std::time::Duration,
    ) -> Result<(), Error> {
        tracing::info!("Auto Commit Delay: {:3.1} s.", delay.as_secs_f32());
        loop {
            tokio::time::delay_for(delay).await;

            let _ = handle
                .write()
                .await
                .commit()
                .map_err(|e| tracing::error!("FileStore: {}", e));
        }

        Ok(())
    }

    // pub async fn read_file(path: impl AsRef<Path>) -> Result<Self> {
    //     let mut lines = BufReader::new(file).lines();

    //     let mut metadata = Vec::with_capacity(lines.size_hint().0);

    //     while let Some(line) = lines.next().await {
    //         let data = serde_json::from_str(&line?)?;
    //         metadata.push(data);
    //     }

    //     Ok(FileStore {
    //         metadata
    //     })
    // }

    // pub fn write_file(&self, path: impl AsRef<Path>) -> Result<()> {
    //     use std::fs::File;
    //     use std::io::{BufWriter, Write};

    //     let file = File::create(path)?;
    //     let mut writer = BufWriter::new(file);

    //     for data in &self.metadata {
    //         serde_json::to_writer(&mut writer, data)?;
    //         writer.write(b"\n")?;
    //     }

    //     writer.flush()?;

    //     Ok(())
    // }
}

impl Drop for FileStore {
    fn drop(&mut self) {
        self.commit().unwrap();
    }
}

fn check_path(path: impl AsRef<Path>) -> Result<(), Error> {
    let file = File::with_options()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)?;

    file.sync_all()?;

    if file.metadata()?.len() == 0 {
        let default_store = FileStore::default();
        default_store.write_file(path)?;
    }

    file.sync_all()?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSettings {
    pub(crate) path: PathBuf,
}

impl Default for StoreSettings {
    fn default() -> Self {
        StoreSettings {
            path: "./store.json".into(),
        }
    }
}
