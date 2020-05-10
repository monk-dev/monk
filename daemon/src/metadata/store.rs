// use std::path::Path;
// use tokio::fs::File;
// use tokio::io::{AsyncBufReadExt, BufReader};
// use tokio::stream::{Stream, StreamExt};

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};

use anyhow::Result;

use super::meta::Meta;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MetaStore(Vec<Meta>);

impl MetaStore {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, meta: Meta) {
        self.0.push(meta);
    }

    pub fn read_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let store = serde_json::from_reader(reader)?;

        Ok(store)
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, self)?;

        Ok(())
    }

    // pub async fn read_file(path: impl AsRef<Path>) -> Result<Self> {
    //     let mut lines = BufReader::new(file).lines();

    //     let mut metadata = Vec::with_capacity(lines.size_hint().0);

    //     while let Some(line) = lines.next().await {
    //         let data = serde_json::from_str(&line?)?;
    //         metadata.push(data);
    //     }

    //     Ok(MetaStore {
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
