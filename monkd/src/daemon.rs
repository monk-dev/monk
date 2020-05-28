use crate::error::Error;
use crate::index::Index;
use crate::metadata::{
    offline_store::{OfflineData, OfflineStore, Status},
    FileStore, Meta,
};
use crate::server::{request::Request, response::Response};
use crate::settings::Settings;

use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Daemon<'s> {
    store: Arc<RwLock<FileStore>>,
    index: Arc<RwLock<Index>>,
    offline: Arc<RwLock<OfflineStore>>,
    settings: &'s Settings,
}

impl<'s> Daemon<'s> {
    pub fn new(settings: &'s Settings) -> Result<Self, Error> {
        let store = Arc::new(RwLock::new(FileStore::read_file(&settings.store().path)?));
        let index = Arc::new(RwLock::new(Index::new(&settings.index())?));
        let offline = Arc::new(RwLock::new(OfflineStore::read_file(
            &settings.offline().path.join("offline.json"),
        )?));

        let store_clone = store.clone();
        let delay = std::time::Duration::from_millis(
            std::cmp::max(settings.daemon().timeout / 3, 3) as u64,
        );

        tokio::spawn(async move {
            tracing::info!("Auto Commit Delay: {:3.1} s.", delay.as_secs_f32());
            loop {
                tokio::time::delay_for(delay).await;

                let _ = store_clone
                    .write()
                    .await
                    .commit()
                    .map_err(|e| tracing::error!("FileStore: {}", e));
            }
        });

        Ok(Self {
            store,
            index,
            offline,
            settings,
        })
    }

    pub async fn handle_request(&mut self, req: Request) -> Result<Response, Error> {
        // let offline = Arc::clone(&self.offline);
        // let store = Arc::clone(&self.store);

        match req {
            Request::Add { name, url, comment } => {
                tracing::info!("[add] {:?} {:?}", name, url);
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
                let id = meta.id().to_string();

                match self.store.write().await.push(meta) {
                    Ok(()) => Ok(Response::NewId(id)),
                    Err(e) => Ok(Response::Error(e.to_string())),
                }
            }
            Request::Delete { id } => {
                tracing::info!("[delete] {:?}", id);

                match self.store.write().await.delete(&id) {
                    Ok(e) => Ok(Response::Item(e)),
                    Err(e) => Ok(Response::Error(e.to_string())),
                }
            }
            Request::List { count } => {
                tracing::info!("[list] {:?}", count);

                if let Some(count) = count {
                    let mut data = self.store.read().await.data().to_vec();
                    data.truncate(count);

                    Ok(Response::List(data))
                } else {
                    Ok(Response::List(self.store.read().await.data().to_vec()))
                }
            }
            Request::Get { id } => match self.store.read().await.get(&id) {
                Some(m) => Ok(Response::Item(m.clone())),
                None => Ok(Response::NotFound(id)),
            },
            Request::Download { id } => {
                // We only want to download a single file:

                if let Some(meta) = self.store.read().await.get(&id).cloned() {
                    if let Some(data) = self.offline.read().await.get(meta.id()) {
                        if data.status == Status::Ready {
                            return Ok(Response::Status(data.id().to_string(), Status::Ready));
                        }
                    }

                    let offline = Arc::clone(&self.offline);
                    tokio::spawn(async move {
                        if let Err(e) = OfflineStore::download_meta(meta, offline).await {
                            tracing::error!("{}", e);
                        }
                    });

                    Ok(Response::Ok)
                } else {
                    Ok(Response::Error(Error::IdNotFound(id).to_string()))
                }
            }
            Request::Open { id } => {
                if let Some(data) = self.offline.read().await.get(&id) {
                    if let Some(path) = &data.file {
                        Ok(Response::Open(path.clone()))
                    } else {
                        Ok(Response::OpenStatus(id, data.status.clone()))
                    }
                } else {
                    Ok(Response::Error(Error::IdNotFound(id).to_string()))
                }
            }
            r => {
                tracing::warn!("Unimplemented Daemon Request: {:?}", r);

                Ok(Response::Unhandled)
            }
        }
    }

    pub async fn shutdown(self) -> Result<(), Error> {
        let store = self.store.write().await;

        if store.is_dirty() {
            store.write_file(&self.settings.store().path)?;
        }

        Ok(())
    }
}
