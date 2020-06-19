use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

use crate::error::Error;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfflineStore {
    data: Vec<OfflineData>,
    pub file: PathBuf,
    dirty: bool,
}

impl OfflineStore {
    pub fn file(&self) -> &Path {
        &self.file
    }

    fn push(&mut self, data: OfflineData) -> Result<(), Error> {
        self.dirty = true;

        if self.get(data.id()).is_err() {
            self.data.push(data);
        }

        Ok(())
        // match self.data.binary_search_by_key(&data.id(), |m| m.id()) {
        //     Ok(_) => Err(Error::AlreadyExists(data.id().to_string())),
        //     Err(index) => {
        //         self.data.insert(index, data);
        //         self.dirty = true;

        //         Ok(())
        //     }
        // }
    }

    pub fn update(&mut self, id: impl AsRef<str>, data: OfflineData) -> Result<(), Error> {
        if id.as_ref() != data.id {
            return Err(Error::UnequalIds);
        }

        match self.get_mut(&id) {
            Ok(d) => *d = data,
            Err(_) => self.push(data)?,
        }

        self.dirty = true;

        Ok(())
    }

    pub fn get(&self, id: impl AsRef<str>) -> Result<&OfflineData, Error> {
        let ids: Vec<usize> = self
            .data
            .iter()
            .enumerate()
            .filter(|(_, m)| m.id().starts_with(id.as_ref()))
            .map(|(i, _)| i)
            .collect();

        // tracing::info!("Ids: {:?}", ids);

        if ids.len() > 1 {
            return Err(Error::TooManyIds(id.as_ref().into(), ids));
        } else if ids.is_empty() {
            return Err(Error::IdNotFound(id.as_ref().into()));
        }

        Ok(&self.data[ids[0]])
    }

    pub fn get_mut(&mut self, id: impl AsRef<str>) -> Result<&mut OfflineData, Error> {
        let ids: Vec<usize> = self
            .data
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

        self.dirty = true;

        Ok(&mut self.data[ids[0]])
    }

    pub fn delete(&mut self, id: impl AsRef<str>) -> Result<OfflineData, Error> {
        let ids: Vec<usize> = self
            .data
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
        let removed = self.data.swap_remove(ids[0]);

        if let Some(file) = &removed.file {
            let _ = std::fs::remove_file(file);
        }

        Ok(removed)
    }

    pub fn read_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        check_path(&path)?;

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        let mut store: OfflineStore = serde_json::from_reader(reader)?;
        store.file = path.as_ref().parent().unwrap().into();
        store.data.sort_by(|l, r| l.id().cmp(r.id()));

        Ok(store)
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn commit(&mut self) -> Result<(), Error> {
        if self.dirty {
            tracing::info!("OfflineStore dirty: {}", self.file().display());

            self.write_file(self.file().join("offline.json"))?;
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

            let _ = handle
                .write()
                .await
                .commit()
                .map_err(|e| tracing::error!("OfflineStore: {}", e));
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfflineData {
    pub id: String,
    pub url: Option<Url>,
    pub file: Option<PathBuf>,
    pub status: Status,
}

impl OfflineData {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Ready,
    Downloading,
    Error(String),
}

impl Status {
    pub fn is_error(&self) -> bool {
        match self {
            Status::Error(_) => true,
            _ => false,
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::Downloading
    }
}

fn check_path(path: impl AsRef<Path>) -> Result<(), Error> {
    use std::fs::OpenOptions;

    let file = OpenOptions::new()
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineSettings {
    pub(crate) data_folder: PathBuf,
    pub(crate) store_file: PathBuf,
}

impl Default for OfflineSettings {
    fn default() -> Self {
        if let Some(dirs) = crate::get_dirs() {
            let data_dir = dirs.data_dir();
            let data_folder = data_dir.join("offline");
            let store_file = data_dir.join("offline.json");

            OfflineSettings {
                data_folder,
                store_file,
            }
        } else {
            OfflineSettings {
                data_folder: "./offline".into(),
                store_file: "offline.json".into(),
            }
        }
    }
}
