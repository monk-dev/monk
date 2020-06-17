use async_channel::Sender;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::oneshot;
use url::Url;

use crate::{
    adapter::Adapter,
    error::Error,
    metadata::{
        monolith,
        offline_store::{OfflineData, OfflineStore, Status},
        Meta,
    },
    Request, Response,
};

#[derive(Debug)]
pub struct HttpAdapter {
    sender: Sender<(Request, Option<oneshot::Sender<Response>>)>,
    in_flight: Arc<AtomicUsize>,
    offline_folder: PathBuf,
}

impl HttpAdapter {
    pub fn new(
        offline_folder: PathBuf,
        sender: Sender<(Request, Option<oneshot::Sender<Response>>)>,
    ) -> Self {
        tracing::info!("Created HTTP Adapter");

        Self {
            sender,
            in_flight: Arc::new(AtomicUsize::new(0)),
            offline_folder,
        }
    }
}

#[async_trait]
impl Adapter for HttpAdapter {
    async fn init_download(
        &mut self,
        meta: Option<&Meta>,
        offline: Option<OfflineData>,
    ) -> Option<OfflineData> {
        if let Some(offline) = offline {
            if offline.status == Status::Ready
                || offline.status.is_error()
                || !valid_url(offline.url.as_ref())
            {
                None
            } else {
                Some(offline)
            }
        } else if let Some(meta) = meta {
            if !valid_url(meta.url()) {
                return None;
            }

            Some(OfflineData {
                id: meta.id().to_string(),
                url: meta.url().cloned(),
                file: None,
                status: Status::Downloading,
            })
        } else {
            None
        }
    }

    async fn handle_download<'s, 'a>(
        &'s mut self,
        meta: Option<&'a Meta>,
        offline: Option<OfflineData>,
    ) -> Option<Result<Response, Error>> {
        if let Some(meta) = meta {
            if !valid_url(meta.url()) {
                return None;
            }

            if meta.url().is_none() {
                return Some(Ok(Response::Error(format!("`{}` has no url", meta.id()))));
            }

            if let Some(ref offline) = offline {
                if offline.status == Status::Ready {
                    return Some(Ok(Response::Status(meta.id().to_string(), Status::Ready)));
                }
            }

            let offline_data = offline.unwrap_or_else(|| OfflineData {
                id: meta.id().to_string(),
                url: meta.url().cloned(),
                file: None,
                status: Status::Downloading,
            });

            let meta = meta.clone();
            let semaphore = Arc::clone(&self.in_flight);
            let sender = self.sender.clone();
            let offline_folder = self.offline_folder.join("offline");

            tokio::spawn(async move {
                semaphore.fetch_add(1, Ordering::SeqCst);
                match download_meta(meta, offline_folder, offline_data).await {
                    Ok(new_data) => {
                        tracing::info!("sending updated offline_data: {:?}", new_data);
                        if let Err(e) = sender.send((Request::UpdateOffline(new_data), None)).await
                        {
                            tracing::error!("{}", e);
                        }
                    }
                    Err(e) => tracing::error!("{}", e),
                }
                semaphore.fetch_sub(1, Ordering::SeqCst);
            });

            Some(Ok(Response::Ok))
        } else {
            None
        }
    }

    async fn shutdown(&mut self) -> Result<(), Error> {
        tracing::info!("Shutting down Http Adapter");

        let in_flight = self.in_flight.load(Ordering::Relaxed);
        if in_flight != 0 {
            tracing::info!("Downloads in flight: {}", in_flight)
        }

        loop {
            if self.in_flight.load(Ordering::Relaxed) == 0 {
                break;
            }
        }

        tracing::info!("Finished shutting down Http Adapter");

        Ok(())
    }
}

async fn download_meta(
    meta: Meta,
    offline_folder: PathBuf,
    mut data: OfflineData,
) -> Result<OfflineData, Error> {
    tracing::info!("[HTTP] download_meta: {:?}", meta.url());

    match tokio::task::spawn_blocking(move || monolith::download_meta(&meta, offline_folder))
        .await?
    {
        Ok(path) => {
            data.status = Status::Ready;
            data.file = Some(path);
        }
        Err(e) => {
            data.status = Status::Error(e.to_string());
        }
    }

    Ok(data)
}

pub fn valid_url(url: Option<&Url>) -> bool {
    if let Some(url) = url {
        url.scheme() != "https" || url.scheme() != "http"
    } else {
        false
    }
}
