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

    pub fn offline_handle(&self) -> Arc<RwLock<OfflineStore>> {
        Arc::clone(&self.offline)
    }

    #[tracing::instrument(skip(self))]
    pub async fn handle_add(
        &mut self,
        name: Option<String>,
        url: Option<url::Url>,
        comment: Option<String>,
    ) -> Result<Response, Error> {
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

        self.store.write().await.push(meta).map(|_| Response::Ok)
    }

    #[tracing::instrument(skip(self))]
    pub async fn handle_download(&mut self, id: String) -> Result<Response, Error> {
        let store = self.store.read().await;
        let meta = store.get(&id)?;

        if let Ok(data) = self.offline.read().await.get(meta.id()) {
            if data.status == Status::Ready {
                return Ok(Response::Status(data.id().to_string(), Status::Ready));
            }
        }

        let offline = self.offline_handle();
        let meta = meta.clone();
        tokio::spawn(async move {
            if let Err(e) = OfflineStore::download_meta(meta, offline).await {
                tracing::error!("{}", e);
            }
        });

        Ok(Response::Ok)
    }

    #[tracing::instrument(skip(self))]
    pub async fn handle_delete(&mut self, id: String) -> Result<Response, Error> {
        self.store.write().await.delete(&id).map(Response::Item)
    }

    #[tracing::instrument(skip(self))]
    pub async fn handle_list(&mut self, count: Option<usize>) -> Result<Response, Error> {
        if let Some(count) = count {
            let mut data = self.store.read().await.data().to_vec();
            data.truncate(count);

            Ok(Response::List(data))
        } else {
            Ok(Response::List(self.store.read().await.data().to_vec()))
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn handle_get(&mut self, id: String) -> Result<Response, Error> {
        match self.store.read().await.get(&id) {
            Ok(m) => Ok(Response::Item(m.clone())),
            Err(e) => match e {
                Error::IdNotFound(id) => Ok(Response::NotFound(id)),
                Error::TooManyIds(id, idxs) => {
                    let store = self.store.read().await;
                    let metas = idxs.into_iter().map(|i| store.index(i).clone()).collect();

                    Ok(Response::TooManyMeta(id, metas))
                }
                e => Ok(Response::Error(e.to_string())),
            },
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn handle_open(&mut self, id: String) -> Result<Response, Error> {
        match self.offline.read().await.get(&id) {
            Ok(data) => {
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
        // let offline = Arc::clone(&self.offline);
        // let store = Arc::clone(&self.store);

        match req {
            Request::Add { name, url, comment } => self.handle_add(name, url, comment).await,
            Request::Delete { id } => self.handle_delete(id).await,
            Request::List { count } => self.handle_list(count).await,
            Request::Get { id } => self.handle_get(id).await,
            Request::Download { id } => self.handle_download(id).await,
            Request::Open { id } => self.handle_open(id).await,
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
