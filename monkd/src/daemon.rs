use crate::error::Error;
use crate::index::Index;
use crate::metadata::{
    offline_store::{OfflineData, OfflineStore, Status},
    FileStore, Meta,
};
use crate::server::{request::Request, response::Response};
use crate::settings::Settings;
use crate::adapter::{Adapter, AdapterSlug, http::HttpAdapter};

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{RwLock, Mutex};
use tokio::task::JoinHandle;
use tracing::info;
use async_lock::Lock;

pub struct Daemon<'s> {
    store: Arc<RwLock<FileStore>>,
    index: Arc<RwLock<Index>>,
    offline: Arc<RwLock<OfflineStore>>,
    settings: &'s Settings,
    adapters: Vec<Lock<Box<dyn Adapter>>>,
}

impl<'s> Daemon<'s> {
    pub fn new(settings: &'s Settings, adapters: Vec<Lock<Box<dyn Adapter>>>) -> Result<Self, Error> {
        let store = Arc::new(RwLock::new(FileStore::read_file(&settings.store().path)?));
        let index = Arc::new(RwLock::new(Index::new(&settings.index())?));
        let offline = Arc::new(RwLock::new(OfflineStore::read_file(
            &settings.offline().path.join("offline.json"),
        )?));

        let store_clone = store.clone();
        let store_delay = std::time::Duration::from_millis(std::cmp::max(
            settings.daemon().timeout / 3,
            3000,
        ) as u64);

        let offline_clone = offline.clone();
        let offline_delay = store_delay.clone();

        tokio::spawn(async move { FileStore::commit_loop(store_clone, store_delay).await });
        tokio::spawn(async move { OfflineStore::commit_loop(offline_clone, offline_delay).await });

        Ok(Self {
            store,
            index,
            offline,
            settings,
            adapters,
        })
    }

    pub fn offline_handle(&self) -> Arc<RwLock<OfflineStore>> {
        Arc::clone(&self.offline)
    }

    pub async fn handle_add(
        &mut self,
        name: Option<String>,
        url: Option<url::Url>,
        comment: Option<String>,
    ) -> Result<Response, Error> {
        info!("[add] {:?} {:?} {:?}", name, url, comment.is_some());
        let mut builder = Meta::builder();

        if let Some(name) = name {
            builder = builder.name(name);
        }

        if let Some(url) = url {
            builder = builder.url(url);
        }

        if let Some(comment) = comment {
            builder = builder.comment(comment);
        }

        let meta = builder.build();

        for adapter in &self.adapters {
            let mut adapter = adapter.lock().await;

            if let Some(res) = adapter.handle_add(&meta).await {
                return res;
            }
        }

        self.store
            .write()
            .await
            .push(meta.clone())
            .map(|_| Response::Item(meta))
    }

    pub async fn handle_download(&mut self, id: Option<String>) -> Result<Response, Error> {
        if let Some(id) = id {
            let store = self.store.read().await;
            let meta = store.get(&id)?;

            for adapter in &mut self.adapters {
                let mut adapter = adapter.lock().await;

                // Get the current offline data if it exists, or initialize it.
                let data  = if let Ok(data) = self.offline.read().await.get(&id) {
                    Some(data.clone())
                } else {
                    adapter.init_download(Some(&meta), None).await
                };
                
                // If this adapter handled the request use it
                if let Some(resp) = adapter.handle_download(Some(&meta), data).await {
                    return resp;
                }
            }
        }

        // No adapter capable of handling the download
        Ok(Response::Unhandled)
    }

    pub async fn handle_delete(&mut self, id: String) -> Result<Response, Error> {
        info!("[delete] {:?}", id);
        let _ = self.offline.write().await.delete(&id);
        self.store.write().await.delete(&id).map(Response::Item)
    }

    pub async fn handle_list(&mut self, count: Option<usize>) -> Result<Response, Error> {
        info!("[list] {:?}", count);
        if let Some(count) = count {
            let mut data = self.store.read().await.data().to_vec();
            data.truncate(count);

            Ok(Response::List(data))
        } else {
            Ok(Response::List(self.store.read().await.data().to_vec()))
        }
    }

    pub async fn handle_get(&mut self, id: String) -> Result<Response, Error> {
        info!("[get] {:?}", id);
        match self.store.read().await.get(&id) {
            Ok(m) => Ok(Response::Item(m.clone())),
            Err(e) => match e {
                Error::IdNotFound(id) => Ok(Response::NotFound(id)),
                e => Ok(Response::Error(e.to_string())),
            },
        }
    }

    pub async fn handle_open(&mut self, id: String) -> Result<Response, Error> {
        info!("[open] {:?}", id);
        match self.offline.read().await.get(&id) {
            Ok(data) => {
                use chrono::Utc;

                {
                    let mut store = self.store.write().await;
                    let meta = store.get_mut(&id)?;

                    let now = Utc::now();
                    meta.last_read = Some(now);
                }

                if let Some(path) = &data.file {
                    Ok(Response::Open(path.clone()))
                } else {
                    Ok(Response::OpenStatus(id, data.status.clone()))
                }
            }
            Err(_e) => Ok(Response::Error(Error::IdNotFound(id).to_string())),
        }
    }

    pub async fn handle_request(&mut self, req: Request) -> Result<Response, Error> {
        tracing::info!("handling request: {:?}", req);

        match req {
            Request::Add { name, url, comment } => self.handle_add(name, url, comment).await,
            Request::Delete { id } => self.handle_delete(id).await,
            Request::List { count } => self.handle_list(count).await,
            Request::Get { id } => self.handle_get(id).await,
            Request::Download { id } => self.handle_download(id).await,
            Request::Open { id } => self.handle_open(id).await,
            Request::UpdateMeta(m) => {
                self.store.write().await.update(m.id().to_string(), m)?;
                Ok(Response::Ok)
            }
            Request::UpdateOffline(o) => {
                self.offline.write().await.update(o.id().to_string(), o)?;
                Ok(Response::Ok)
            }
            r => {
                tracing::warn!("Unimplemented Daemon Request: {:?}", r);
                Ok(Response::Unhandled)
            }
        }
    }

    pub async fn shutdown(self) -> Result<(), Error> {
        // Commit any changes to the store
        self.store.write().await.commit()?;
        self.offline.write().await.commit()?;

        Ok(())
    }
}

