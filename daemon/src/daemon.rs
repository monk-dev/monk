use crate::error::Error;
use crate::metadata::{FileStore, Meta};
use crate::server::{request::Request, response::Response};
use crate::settings::Settings;

use tokio::sync::RwLock;

pub struct Daemon<'s> {
    store: RwLock<FileStore>,
    settings: &'s Settings,
}

impl<'s> Daemon<'s> {
    pub fn new(settings: &'s Settings) -> Result<Self, Error> {
        let store = RwLock::new(FileStore::read_file(&settings.store())?);

        Ok(Self { store, settings })
    }

    pub async fn handle_request(&mut self, req: Request) -> Result<Response, Error> {
        match req {
            Request::Add { name, url } => {
                tracing::info!("[add] {} {:?}", name, url);
                let mut builder = Meta::builder(name);
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
            Request::Stop => {
                tracing::info!("Daemon cannot handle Request::Stop");

                Ok(Response::Ok)
            }
            r => {
                tracing::warn!("Unimplemented Request: {:?}", r);

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
