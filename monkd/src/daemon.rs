use crate::adapter::Adapter;
use crate::error::Error;
use crate::index::Index;
use crate::metadata::{
    meta::IndexStatus,
    offline_store::{OfflineStore, Status as OfflineStatus},
    FileStore, Meta,
};
use crate::server::{
    request::{Edit, Request, StatusKind},
    response::Response,
};
use crate::settings::Settings;
use crate::status::*;

use async_channel::Sender;
use async_lock::Lock;
use std::sync::Arc;
use tokio::sync::RwLock;

use tracing::{error, info};

pub struct Daemon<'s> {
    store: Arc<RwLock<FileStore>>,
    index: Arc<RwLock<Index>>,
    offline: Arc<RwLock<OfflineStore>>,
    daemon_sender: Sender<(Request, Option<tokio::sync::oneshot::Sender<Response>>)>,
    adapters: Vec<Lock<Box<dyn Adapter>>>,
    settings: &'s Settings,
}

impl<'s> Daemon<'s> {
    pub fn new(
        settings: &'s Settings,
        daemon_sender: Sender<(Request, Option<tokio::sync::oneshot::Sender<Response>>)>,
        adapters: Vec<Lock<Box<dyn Adapter>>>,
    ) -> Result<Self, Error> {
        let store = Arc::new(RwLock::new(FileStore::read_file(&settings.store().path)?));
        let index = Arc::new(RwLock::new(Index::new(&settings.index())?));
        let offline = Arc::new(RwLock::new(OfflineStore::read_file(
            &settings.offline().store_file,
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
            daemon_sender,
            adapters,
            settings,
        })
    }

    pub fn offline_handle(&self) -> Arc<RwLock<OfflineStore>> {
        Arc::clone(&self.offline)
    }

    pub async fn handle_status(&self, kind: StatusKind) -> Result<Response, Error> {
        tracing::info!("[status] {:?}", kind);

        match kind {
            StatusKind::All => {
                let file_store = self.store.read().await;
                let offline_store = self.offline.read().await;
                let index = self.index.read().await;

                let file_store_status = FileStoreStatus::new(&file_store)?;
                let offline_status = OfflineStoreStatus::new(&offline_store)?;
                let index_status = TIndexStatus::new(&index)?;

                let status = StatusResponse {
                    meta: None,
                    file_store: Some(file_store_status),
                    offline_store: Some(offline_status),
                    index_status: Some(index_status),
                };

                Ok(Response::Status(status))
            }
            StatusKind::Index => {
                let index = self.index.read().await;
                let index_status = TIndexStatus::new(&index)?;

                Ok(Response::Status(StatusResponse {
                    meta: None,
                    file_store: None,
                    offline_store: None,
                    index_status: Some(index_status),
                }))
            }
            StatusKind::Offline => {
                let offline = self.offline.read().await;
                let offline_status = OfflineStoreStatus::new(&offline)?;

                Ok(Response::Status(StatusResponse {
                    meta: None,
                    file_store: None,
                    offline_store: Some(offline_status),
                    index_status: None,
                }))
            }
            StatusKind::Store => {
                let store = self.store.read().await;
                let store_status = FileStoreStatus::new(&store)?;

                Ok(Response::Status(StatusResponse {
                    meta: None,
                    file_store: Some(store_status),
                    offline_store: None,
                    index_status: None,
                }))
            }
            StatusKind::Id(id) => {
                let store = self.store.read().await;
                let offline = self.offline.read().await;
                let meta = match store.get(&id) {
                    Err(_) => return Ok(Response::NotFound(id)),
                    Ok(m) => m,
                };

                let meta_status = MetaStatus::new(&meta, &offline)?;

                Ok(Response::Status(StatusResponse {
                    meta: Some(meta_status),
                    file_store: None,
                    offline_store: None,
                    index_status: None,
                }))
            }
        }
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
        let mut _adapter_result = None;

        for adapter in &self.adapters {
            let mut adapter = adapter.lock().await;

            if let Some(res) = adapter.handle_add(&meta).await {
                _adapter_result = Some(res);
                break;
            }
        }

        if self.settings.daemon().download_after_add {
            info!("auto downloading: [{}]", meta.id());

            let download_req = Request::Download {
                id: Some(meta.id().to_string()),
            };

            let _ = self
                .daemon_sender
                .send((download_req, None))
                .await
                .map_err(|_| error!("error automatically downloading [{}]", meta.id()));
        }

        // TODO: Use `Response::Many` for these errors
        // e.g. We still want to add to store even if
        // indexing fails.
        let _ = self.index.write().await.insert_meta(&meta);

        self.store
            .write()
            .await
            .push(meta.clone())
            .map(|_| Response::Item(meta.clone()))
    }

    pub async fn handle_index(&mut self, id: String) -> Result<Response, Error> {
        let mut meta = {
            let mut store = self.store.write().await;

            let mut meta = store.get_mut(&id)?;
            meta.index_status = Some(IndexStatus::Indexing);

            meta.clone()
        };

        let offline = { self.offline.read().await.get(&id).ok().cloned() };

        for adapter in &mut self.adapters {
            if adapter.lock().await.will_index(&meta, offline.as_ref()) {
                let adapter = adapter.clone();
                let daemon_sender = self.daemon_sender.clone();
                let index = self.index.clone();

                tokio::spawn(async move {
                    let mut index = index.write().await;
                    let mut adapter = adapter.lock().await;

                    if let Some(result) = adapter
                        .handle_index(&meta, offline.as_ref(), &mut index)
                        .await
                    {
                        // This meta was successfully indexed by this adapter:
                        if result.is_ok() {
                            tracing::info!("[{}] indexed", meta.id());
                            meta.index_status = Some(IndexStatus::Indexed);
                            let _ = daemon_sender.send((Request::UpdateMeta(meta), None)).await;
                        }
                    }
                });

                return Ok(Response::Ok);
            }
        }

        Ok(Response::NoAdapterFound(meta.id().to_string()))
    }

    pub async fn handle_index_status(&self, id: String) -> Result<Response, Error> {
        Ok(Response::IndexStatus(
            id.clone(),
            self.store.read().await.get(&id)?.index_status.clone(),
        ))
    }

    pub async fn handle_index_all(&mut self) -> Result<Response, Error> {
        let mut response = Vec::new();

        let ids: Vec<_> = {
            let store = self.store.read().await;
            store.data().iter().map(|m| m.id().to_string()).collect()
        };

        for id in ids {
            if let Err(e) = self.handle_index(id).await {
                response.push(Response::Error(e.to_string()));
            }
        }

        if !response.is_empty() {
            Ok(Response::Many(response))
        } else {
            Ok(Response::Indexing("all".to_string()))
        }
    }

    pub async fn handle_download(&mut self, id: Option<String>) -> Result<Response, Error> {
        if let Some(id) = id {
            let store = self.store.read().await;
            let meta = store.get(&id)?;

            for adapter in &mut self.adapters {
                let mut adapter = adapter.lock().await;

                // Get the current offline data if it exists, or initialize it.
                let data = if let Ok(data) = self.offline.read().await.get(&id) {
                    Some(data.clone())
                } else {
                    adapter.init_download(Some(&meta), None).await
                };

                // If this adapter handled the request use it
                if let Some(resp) = adapter.handle_download(Some(&meta), data).await {
                    return resp;
                }
            }

            Ok(Response::Unhandled)
        } else {
            let offline_store = self.offline.read().await;

            // Download everything in the store:
            let ids: Vec<String> = self
                .store
                .read()
                .await
                .data()
                .iter()
                .filter(|m| offline_store.get(m.id()).is_err())
                .map(|d| d.id().to_string())
                .collect();

            let count = ids.len();

            for id in ids {
                let req = Request::Download { id: Some(id) };
                let _ = self
                    .daemon_sender
                    .send((req, None))
                    .await
                    .map_err(|_| error!("error sending download req"));
            }

            match count {
                0 => Ok(Response::Custom(format!("no items to download"))),
                1 => Ok(Response::Custom(format!("downloading 1 item"))),
                count => Ok(Response::Custom(format!("downloading {} items", count))),
            }
        }
    }

    pub async fn handle_edit(&mut self, id: String, edit: Edit) -> Result<Response, Error> {
        info!("[edit] {:?}", edit);

        let _ = self.offline.write().await.edit(&id, &edit);

        if let Some(_) = edit.url {
            let req = Request::Download {
                id: Some(id.to_string()),
            };
            let _ = self
                .daemon_sender
                .send((req, None))
                .await
                .map_err(|_| error!("error sending download req"));
        }

        //This will respond with an ok or an error
        self.store
            .write()
            .await
            .edit(&id, &edit)
            .map(Response::Item)
    }

    pub async fn handle_delete(&mut self, id: String) -> Result<Response, Error> {
        info!("[delete] {:?}", id);
        let id = { self.store.read().await.get(id)?.id().to_string() };

        let _ = self.offline.write().await.delete(&id);
        let _ = self.index.write().await.delete(&id);

        // We currently only really care if it was in the store.
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

    pub async fn handle_search(
        &mut self,
        query: String,
        count: Option<usize>,
    ) -> Result<Response, Error> {
        let count = count.unwrap_or(1).max(1);
        let ids = self.index.write().await.search(query, count)?;
        let mut metas = Vec::new();

        for id in ids {
            let meta = self.store.read().await.get(id)?.clone();
            metas.push(meta)
        }

        Ok(Response::SearchResult(metas))
    }

    pub async fn handle_open(&mut self, id: String, online: bool) -> Result<Response, Error> {
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

                if online {
                    if let Some(url) = &data.url {
                        return Ok(Response::Open(std::path::PathBuf::from(
                            url.clone().to_string(),
                        )));
                    } else {
                        return Ok(Response::Error("No Url Found".to_string()));
                    }
                }

                if let Some(path) = &data.file {
                    Ok(Response::Open(path.clone()))
                } else {
                    Ok(Response::OpenStatus(
                        data.id().to_string(),
                        data.status.clone(),
                    ))
                }
            }
            Err(_e) => {
                let store = self.store.read().await;
                if self.settings.daemon().download_on_open && store.get(&id).is_ok() {
                    let req = Request::Download {
                        id: Some(id.to_string()),
                    };
                    let _ = self
                        .daemon_sender
                        .send((req, None))
                        .await
                        .map_err(|_| error!("error sending download req"));

                    Ok(Response::MetaOfflineStatus(
                        id.to_string(),
                        OfflineStatus::Downloading,
                    ))
                } else {
                    Ok(Response::NotFound(id.to_string()))
                }
            }
        }
    }

    pub async fn handle_request(&mut self, req: Request) -> Result<Response, Error> {
        tracing::info!("handling request: {:?}", req);

        match req {
            Request::Add { name, url, comment } => self.handle_add(name, url, comment).await,
            Request::Edit { id, edit } => self.handle_edit(id, edit).await,
            Request::Delete { id } => self.handle_delete(id).await,
            Request::List { count } => self.handle_list(count).await,
            Request::Get { id } => self.handle_get(id).await,
            Request::Download { id } => self.handle_download(id).await,
            Request::Open { id, online } => self.handle_open(id, online).await,
            Request::UpdateMeta(m) => {
                self.store.write().await.update(m.id().to_string(), m)?;
                Ok(Response::Ok)
            }
            Request::UpdateOffline(o) => {
                self.offline.write().await.update(o.id().to_string(), o)?;
                Ok(Response::Ok)
            }
            Request::Index { id } => self.handle_index(id).await,
            Request::IndexStatus { id } => self.handle_index_status(id).await,
            Request::IndexAll => self.handle_index_all().await,
            Request::Status { kind } => self.handle_status(kind).await,
            Request::Search { count, query } => self.handle_search(query, count).await,
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
