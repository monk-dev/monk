use crate::error::Error;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use url::Url;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfflineStore {
    data: Vec<OfflineData>,
    file: PathBuf,
    dirty: bool,
}

impl OfflineStore {
    pub fn file(&self) -> &Path {
        &self.file
    }

    pub fn read_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        check_path(&path)?;

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        let mut store: OfflineStore = serde_json::from_reader(reader)?;
        store.file = path.as_ref().into();
        store.data.sort_by(|l, r| l.id().cmp(r.id()));

        Ok(store)
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, self)?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn commit(&mut self) -> Result<(), Error> {
        if self.dirty {
            tracing::info!("OfflineStore dirty: {}", self.file().display());

            self.write_file(self.file())?;
            self.dirty = false;
        } else {
            tracing::info!("OfflineStore clean: {}", self.file().display());
        }

        Ok(())
    }

    pub async fn commit_loop(
        handle: Arc<RwLock<OfflineStore>>,
        delay: std::time::Duration,
    ) -> Result<(), Error> {
        tracing::info!("Auto Commit Delay: {:3.1} s.", delay.as_secs_f32());
        loop {
            tokio::time::delay_for(delay).await;

            handle
                .write()
                .await
                .commit()
                .map_err(|e| tracing::error!("OfflineStore: {}", e));
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfflineData {
    id: String,
    url: Option<Url>,
    file: Option<PathBuf>,
    dirty: bool,
}

impl OfflineData {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    // pub fn is_indexed(&self) -> bool {
    //     self.indexed
    // }

    pub fn is_dirty(&self) -> bool {
        self.dirty
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
        let default_store = OfflineStore::default();
        default_store.write_file(path)?;
    }

    file.sync_all()?;

    Ok(())
}
