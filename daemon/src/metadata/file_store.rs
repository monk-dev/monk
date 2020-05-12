// use std::path::Path;
// use tokio::fs::File;
// use tokio::io::{AsyncBufReadExt, BufReader};
// use tokio::stream::{Stream, StreamExt};

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

use super::meta::Meta;
use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

    pub fn push(&mut self, meta: Meta) {
        self.dirty = true;
        self.metadata.push(meta);
    }

    pub fn read_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        let mut store: FileStore = serde_json::from_reader(reader)?;
        store.file = Some(path.as_ref().into());

        Ok(store)
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, self)?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn commit(&mut self) -> Result<()> {
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
