// use std::path::Path;
// use tokio::fs::File;
// use tokio::io::{AsyncBufReadExt, BufReader};
// use tokio::stream::{Stream, StreamExt};

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, instrument};

use crate::server::request::Edit;

use super::Meta;
use crate::error::Error;

pub const CURRENT_FILE_STORE_VERSION: &'static str = "0.1.0";

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FileStore {
    version: String,
    metadata: Vec<Meta>,
    #[serde(skip)]
    file: PathBuf,
    #[serde(skip)]
    dirty: bool,
    tags: BTreeMap<String, BTreeSet<String>>, // tag -> id
}

impl FileStore {
    pub fn empty() -> Self {
        Self {
            version: CURRENT_FILE_STORE_VERSION.to_string(),
            metadata: Vec::new(),
            file: PathBuf::new(),
            dirty: false,
            tags: BTreeMap::new(),
        }
    }

    pub fn push(&mut self, meta: Meta) -> Result<(), Error> {
        self.dirty = true;
        if !meta.tags.is_empty() {
            for tag in meta.tags.iter() {
                if let Some(ids) = self.tags.get_mut(tag) {
                    ids.insert(meta.id.clone());
                } else {
                    let mut id_set = BTreeSet::new();
                    id_set.insert(meta.id.clone());
                    self.tags.insert(tag.clone(), id_set);
                }
            }
        }
        self.metadata.push(meta);
        Ok(())
    }

    pub fn file(&self) -> &Path {
        &self.file
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn tags(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.tags
    }

    pub fn read_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        check_path(&path)?;

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        let mut store: FileStore = serde_json::from_reader(reader)?;
        store.file = path.as_ref().into();
        // store.metadata.sort_by(|l, r| l.id().cmp(r.id()));

        Ok(store)
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)?;

        Ok(())
    }

    pub fn get(&self, description: impl AsRef<str>) -> Result<&Meta, Error> {
        let id = self.find_id(&description)?;
        Ok(&self.metadata[id])
    }

    pub fn get_mut(&mut self, description: impl AsRef<str>) -> Result<&mut Meta, Error> {
        let id = self.find_id(&description)?;

        self.dirty = true;

        // self.metadata.iter().find(|m| m.id() == id.as_ref())
        Ok(&mut self.metadata[id])
    }

    // Will not deduplicate if 2 of the same id is passed in
    pub fn get_list<T>(&self, ids: T) -> Vec<Meta>
    where
        T: IntoIterator,
        T::Item: AsRef<str>,
    {
        let mut metas = Vec::new();
        for id in ids {
            match self.get(id) {
                Ok(m) => metas.push(m.clone()),
                _ => (),
            };
        }
        metas
    }

    // Takes in a collection of tags and returns all metas that have at least
    // one of the tags in the list. Think union.
    pub fn get_union_tags<T>(&self, tags: T) -> Vec<Meta>
    where
        T: IntoIterator,
        T::Item: ToString,
    {
        let mut ids: BTreeSet<String> = BTreeSet::new();
        for tag in tags {
            if let Some(m) = self.tags().get(&tag.to_string()) {
                for id in m {
                    ids.insert(id.clone());
                }
            }
        }
        self.get_list(ids)
    }

    // Takes in a colection of tags and returns all metas that have all
    // tags passed in. think intersect
    pub fn get_intersection_tags<T>(&self, tags: T) -> Vec<Meta>
    where
        T: IntoIterator,
        T::Item: ToString,
    {
        let mut ids: BTreeSet<String> = BTreeSet::new();
        let mut is_first = true;

        for tag in tags {
            if is_first {
                if let Some(m) = self.tags().get(&tag.to_string()) {
                    for id in m {
                        ids.insert(id.clone());
                    }
                }
                is_first = false;
            }
            if let Some(m) = self.tags().get(&tag.to_string()) {
                ids = ids.intersection(&m).cloned().collect();
            }
        }
        self.get_list(ids)
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

    pub fn edit(&mut self, description: &impl AsRef<str>, edit: &Edit) -> Result<Meta, Error> {
        let id: usize = self.find_id(&description)?;

        tracing::info!("Editing: {:?}", edit);
        self.dirty = true;

        if let Some(u) = edit.url.as_ref() {
            self.metadata[id].url = Some(url::Url::parse(&u)?)
        }
        if let Some(n) = edit.name.as_ref() {
            self.metadata[id].name = Some(n.clone());
        }
        if let Some(c) = edit.comment.as_ref() {
            self.metadata[id].comment = Some(c.clone());
        }
        for tag in edit.add_tags.iter() {
            if let Some(ids) = self.tags.get_mut(tag) {
                ids.insert(self.metadata[id].id.clone());
            } else {
                let mut id_set = BTreeSet::new();
                id_set.insert(self.metadata[id].id.clone());
                self.tags.insert(tag.clone(), id_set);
            }
            self.metadata[id].tags.insert(tag.clone());
        }
        for tag in edit.remove_tags.iter() {
            if let Some(ids) = self.tags.get_mut(tag) {
                ids.remove(&self.metadata[id].id);
            }
            self.metadata[id].tags.remove(tag);
        }

        Ok(self.metadata[id].clone())
    }

    pub fn delete(&mut self, description: impl AsRef<str>) -> Result<Meta, Error> {
        let id = self.find_id(&description)?;

        tracing::info!("Deleting: `{}`", description.as_ref());
        self.dirty = true;

        let removed = self.metadata.swap_remove(id);

        // House keeping for tag -> metadata data structure
        for tag in removed.tags.iter() {
            if let Some(set) = self.tags.get_mut(tag) {
                set.remove(&removed.id);
            }
        }

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
    }

    fn find_id(&self, description: &impl AsRef<str>) -> Result<usize, Error> {
        let ids: Vec<usize> = self
            .metadata
            .iter()
            .enumerate()
            .filter_map(|(i, meta)| {
                if meta.id().starts_with(description.as_ref())
                    || meta
                        .name()
                        .map(|name| name.starts_with(description.as_ref()))
                        .unwrap_or_default()
                {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        tracing::info!("Ids: {:?}", ids);

        if ids.len() > 1 {
            return Err(Error::TooManyMetas(
                description.as_ref().into(),
                ids.into_iter().map(|i| self.metadata[i].clone()).collect(),
            ));
        } else if ids.is_empty() {
            return Err(Error::IdNotFound(description.as_ref().into()));
        }
        Ok(ids[0])
    }
}

impl Drop for FileStore {
    fn drop(&mut self) {
        let _ = self.commit();
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
        let mut default_store = FileStore::default();
        default_store.version = CURRENT_FILE_STORE_VERSION.to_string();
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
        if let Some(dirs) = crate::get_dirs() {
            StoreSettings {
                path: dirs.data_dir().join("store.json"),
            }
        } else {
            StoreSettings {
                path: "./store.json".into(),
            }
        }
    }
}
