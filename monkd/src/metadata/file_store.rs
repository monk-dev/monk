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
use tracing::{debug, info, instrument};

use crate::server::request::Edit;

use super::Meta;
use crate::error::Error;

pub const CURRENT_FILE_STORE_VERSION: &str = "0.1.0";

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FileStore {
    version: String,
    metadata: Vec<Meta>,
    #[serde(skip)]
    file: PathBuf,
    #[serde(skip)]
    dirty: bool,
    #[serde(default)]
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

    #[instrument(level = "debug", skip(self))]
    pub fn push(&mut self, meta: Meta) {
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

    #[instrument(level = "debug", fields(path))]
    pub fn read_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        check_path(&path)?;
        debug!("Reading FileStore file");

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        let mut store: FileStore = serde_json::from_reader(reader)?;
        store.file = path.as_ref().into();

        // This is more me just playing around with tracing
        tracing::Span::current().record("path", &format!("{:?}", store.file).as_str());

        Ok(store)
    }

    #[instrument(level = "debug", skip(self, path))]
    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        dbg!(path.as_ref());
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)?;

        Ok(())
    }

    #[instrument(level = "debug", skip(self), fields(description = description.as_ref()))]
    pub fn get(&self, description: impl AsRef<str>) -> Result<&Meta, Error> {
        let id = self.find_id(&description)?;
        Ok(&self.metadata[id])
    }

    #[instrument(level = "debug", skip(self), fields(description = description.as_ref()))]
    pub fn get_mut(&mut self, description: &impl AsRef<str>) -> Result<&mut Meta, Error> {
        let id = self.find_id(&description)?;

        self.dirty = true;

        Ok(&mut self.metadata[id])
    }

    // Will not deduplicate if 2 of the same id is passed in
    #[instrument(level = "debug", skip(self, ids))]
    pub fn get_list<T>(&self, ids: T) -> Vec<Meta>
    where
        T: IntoIterator,
        T::Item: AsRef<str>,
    {
        let mut metas = Vec::new();
        for id in ids {
            if let Ok(m) = self.get(id) {
                metas.push(m.clone())
            }
        }
        metas
    }

    // Takes in a collection of tags and returns all metas that have at least
    // one of the tags in the list. Think union.
    #[instrument(level = "debug", skip(self, tags))]
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
    #[instrument(level = "debug", skip(self, tags))]
    pub fn get_intersection_tags<T>(&self, tags: T) -> Vec<Meta>
    where
        T: IntoIterator,
        T::Item: ToString,
    {
        debug!("Getting metas with intersection of tags");
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

    #[instrument(level = "debug")]
    pub fn index(&self, idx: usize) -> &Meta {
        &self.metadata[idx]
    }

    #[instrument(level = "debug", skip(self), fields(id = id.as_ref()))]
    pub fn update(&mut self, id: &impl AsRef<str>, data: Meta) -> Result<(), Error> {
        if id.as_ref() != data.id {
            return Err(Error::UnequalIds);
        }

        let meta = self.get_mut(&id)?;
        *meta = data;

        self.dirty = true;

        Ok(())
    }

    #[instrument(level = "debug", skip(self), fields(description = description.as_ref()))]
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

    #[instrument(level = "debug", skip(self), fields(description = description.as_ref()))]
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
            tokio::time::sleep(delay).await;

            let _ = handle
                .write()
                .await
                .commit()
                .map_err(|e| tracing::error!("FileStore: {}", e));
        }
    }

    #[instrument(level = "debug", skip(self), fields(description = description.as_ref()))]
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

    #[instrument(level = "debug", skip(self))]
    pub fn import_file(&mut self, file: String) -> Result<(), Error> {
        tracing::info!("Importing file {}", file);
        let fp = Path::new(&file);
        match FileStore::read_file(fp) {
            Ok(store) => self.import(store),
            Err(e) => {
                tracing::warn!("Import error: {:?}", e);
                Err(e)
            }
        }
    }

    // This does book keeping for the tag -> id store.
    // I can't wait to change monk to use a relational database
    #[instrument(level = "debug", skip(self))]
    fn union_metas(&mut self, current_meta_id: String, incoming_meta: Meta) -> Result<(), Error> {
        let current_meta = self.get(&current_meta_id)?;
        let new_tags: Vec<_> = incoming_meta
            .tags
            .difference(&current_meta.tags)
            .cloned()
            .collect();

        for tag in new_tags {
            if let Some(ids) = self.tags.get_mut(&tag) {
                ids.insert(current_meta_id.clone());
            } else {
                let mut id_set = BTreeSet::new();
                id_set.insert(current_meta_id.clone());
                self.tags.insert(tag.clone(), id_set);
            }
        }
        let current_meta = self.get_mut(&current_meta_id).unwrap();
        current_meta.union(&incoming_meta);
        Ok(())
    }
    // TODO: This is a target for when monk gets a relational database
    // a url -> Id table would be better than this O(nm) iteration here.
    // Review note, is there something more idiomatic than mut fs
    #[instrument(level = "debug", skip(self))]
    pub fn import(&mut self, mut fs: FileStore) -> Result<(), Error> {
        debug!("Importing Filestore");
        for meta in fs.metadata.drain(..) {
            let mut is_unioned = false;
            let mut id = String::new();
            for m in self.metadata.iter_mut() {
                if meta.url() == m.url() {
                    id = m.id.clone();
                    is_unioned = true;
                    break;
                }
            }
            if !is_unioned {
                self.push(meta);
            } else {
                self.union_metas(id, meta)?;
            }
        }
        Ok(())
    }
}

impl std::ops::Index<usize> for FileStore {
    type Output = Meta;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.metadata[idx]
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
