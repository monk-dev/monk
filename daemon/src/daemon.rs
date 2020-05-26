use crate::error::Error;
use crate::metadata::{FileStore, Meta, OfflineData, OfflineStore};
use crate::server::{request::Request, response::Response};
use crate::settings::Settings;

use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Daemon<'s> {
    store: Arc<RwLock<FileStore>>,
    offline: Arc<RwLock<OfflineStore>>,
    settings: &'s Settings,
}

impl<'s> Daemon<'s> {
    pub fn new(settings: &'s Settings) -> Result<Self, Error> {
        let store = Arc::new(RwLock::new(FileStore::read_file(&settings.store())?));
        let offline = Arc::new(RwLock::new(OfflineStore::read_file(&settings.offline())?));

        let store_clone = store.clone();
        let delay =
            std::time::Duration::from_millis(std::cmp::max(settings.timeout() / 3, 3) as u64);

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

        Ok(Self { store, settings })
    }

    pub async fn handle_request(&mut self, req: Request) -> Result<Response, Error> {
        match req {
            Request::Add { name, url } => {
                tracing::info!("[add] {:?} {:?}", name, url);
                let mut builder = Meta::builder();

                if let Some(name) = name {
                    builder = builder.name(name);
                }

                if let Some(url) = url {
                    builder = builder.url(url);
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
                    Some(e) => Ok(Response::Item(e)),
                    None => Ok(Response::Error(format!("Item with id: {}, not found", id))),
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
            r => {
                tracing::warn!("Unimplemented Daemon Request: {:?}", r);

                Ok(Response::Unhandled)
            }
        }
    }

    pub async fn shutdown(self) -> Result<(), Error> {
        let store = self.store.write().await;

        if store.is_dirty() {
            store.write_file(self.settings.store())?;
        }

        Ok(())
    }
}
