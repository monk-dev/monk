// use std::path::Path;
// use tokio::fs::File;
// use tokio::io::{AsyncBufReadExt, BufReader};
// use tokio::stream::{Stream, StreamExt};

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

use super::meta::Meta;
use crate::error::Error;

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FileStore {
    metadata: Vec<Meta>,
    #[serde(skip)]
    file: Option<PathBuf>,
    #[serde(skip)]
    dirty: bool,
}

impl FileStore {
    pub fn empty() -> Self {
        Self {
            metadata: Vec::new(),
            file: None,
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

    pub fn read_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        check_path(&path)?;

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        let mut store: FileStore = serde_json::from_reader(reader)?;
        store.file = Some(path.as_ref().into());
        store.metadata.sort_by(|l, r| l.id().cmp(r.id()));

        Ok(store)
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, self)?;

        Ok(())
    }

    pub fn get(&self, id: impl AsRef<str>) -> Option<&Meta> {
        self.metadata.iter().find(|m| m.id() == id.as_ref())
    }

    pub fn get_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Meta> {
        let e = self.metadata.iter_mut().find(|m| m.id() == id.as_ref());

        if e.is_some() {
            self.dirty = true;
        }

        e
    }

    pub fn delete(&mut self, id: impl AsRef<str>) -> Option<Meta> {
        let (idx, _) = self
            .metadata
            .iter()
            .enumerate()
            .find(|(_, m)| m.id() == id.as_ref())?;

        self.dirty = true;

        Some(self.metadata.swap_remove(idx))
    }

    pub fn data(&self) -> &[Meta] {
        &self.metadata
    }

    #[instrument(skip(self))]
    pub fn commit(&mut self) -> Result<(), Error> {
        if self.dirty {
            info!("FileStore dirty");
            let path = self.file.as_ref().ok_or_else(|| Error::FileStoreNoPath)?;
            self.write_file(path)?;

            self.dirty = false;
        } else {
            info!("FileStore clean");
        }

        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
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

fn check_path(path: impl AsRef<Path>) -> Result<(), Error> {
    let file = File::with_options()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)?;

    file.sync_all()?;

    if file.metadata()?.len() == 0 {
        let default_store = FileStore::default();
        default_store.write_file(path);
    }

    file.sync_all()?;

    Ok(())
}

impl Drop for FileStore {
    fn drop(&mut self) {
        self.commit().unwrap();
    }
}
