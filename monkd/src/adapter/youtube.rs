use async_channel::Sender;
use async_trait::async_trait;
use std::fs::{create_dir_all, read_dir};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::oneshot;
use tracing::{info, instrument};
use url::Url;

use crate::{
    adapter::{Adapter, AdapterType},
    error::Error,
    index::Index,
    metadata::{
        offline_store::{OfflineData, Status},
        Meta,
    },
    Request, Response,
};

#[derive(Debug)]
pub struct YoutubeAdapter {
    sender: Sender<(Request, Option<oneshot::Sender<Response>>)>,
    in_flight: Arc<AtomicUsize>,
    offline_folder: PathBuf,
}

impl YoutubeAdapter {
    pub fn new(
        offline_folder: PathBuf,
        sender: Sender<(Request, Option<oneshot::Sender<Response>>)>,
    ) -> Self {
        info!("[Youtube] Created Youtube Adapter");
        Self {
            sender,
            in_flight: Arc::new(AtomicUsize::new(0)),
            offline_folder,
        }
    }

    pub fn yt_dl_installed() -> bool {
        Command::new("youtube-dl")
            .arg("-h")
            .stdout(Stdio::null())
            .spawn()
            .is_ok()
    }
}

#[async_trait]
impl Adapter for YoutubeAdapter {
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
                name: meta.name().map(ToOwned::to_owned),
                url: meta.url().cloned(),
                file: None,
                adapter: AdapterType::Youtube,
                status: Status::Downloading,
            })
        } else {
            None
        }
    }

    #[instrument(skip(self, offline))]
    async fn handle_download<'s, 'a>(
        &'s mut self,
        meta: Option<&'a Meta>,
        offline: Option<OfflineData>,
    ) -> Option<Result<Response, Error>> {
        if let Some(meta) = meta {
            if !valid_url(meta.url()) {
                return None;
            }

            if let Some(ref offline) = offline {
                if offline.status == Status::Ready {
                    return Some(Ok(Response::MetaOfflineStatus(
                        meta.id().to_string(),
                        Status::Ready,
                    )));
                }
            }

            let offline_data = offline.unwrap_or_else(|| OfflineData {
                id: meta.id().to_string(),
                name: meta.name().map(ToOwned::to_owned),
                url: meta.url().cloned(),
                file: None,
                adapter: AdapterType::Youtube,
                status: Status::Downloading,
            });

            let meta = meta.clone();
            let semaphore = Arc::clone(&self.in_flight);
            let sender = self.sender.clone();
            let offline_folder = self.offline_folder.clone();

            tokio::spawn(async move {
                semaphore.fetch_add(1, Ordering::SeqCst);
                match download_meta(meta, offline_folder, offline_data).await {
                    Ok(new_data) => {
                        info!("sending updated offline_data: {:?}", new_data);
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

    fn can_modify(&self, meta: &Meta, offline: Option<&OfflineData>) -> bool {
        if let Some(data) = offline {
            return data.adapter == AdapterType::Youtube;
        }
        valid_url(meta.url()) && offline.map(|o| o.file().is_some()).unwrap_or_default()
    }

    fn score_meta(&self, meta: &Meta) -> usize {
        info!("[Youtube] score name: {:?}  url: {:?}", meta.name, meta.url);
        if valid_url(meta.url()) {
            info!("5");
            5
        } else {
            info!("0");
            0
        }
    }

    #[instrument(skip(self, index))]
    async fn handle_index(
        &mut self,
        meta: &Meta,
        offline: Option<&OfflineData>,
        index: &mut Index,
    ) -> Option<Result<(), Error>> {
        let offline = offline?;
        // TODO: make this less janky
        let path = offline.file()?.parent()?.join(meta.id.clone() + ".en.vtt");

        let data = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Could not read file into string {:?}", path);
                return Some(Err(e.into()));
            }
        };

        // At this point there is data to be parsed,
        // so we delete whatever is in the current index
        // and re-add the meta item and data.
        if let Err(e) = index.delete(meta.id()) {
            return Some(Err(e));
        }

        Some(
            index
                .insert_meta_with_data(meta, meta.name(), Some(&data), None)
                .map(|_| ()),
        )
    }

    #[instrument(skip(self))]
    async fn shutdown(&mut self) -> Result<(), Error> {
        let in_flight = self.in_flight.load(Ordering::Relaxed);
        if in_flight != 0 {
            info!("Downloads in flight: {}", in_flight)
        }

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            if self.in_flight.load(Ordering::Relaxed) == 0 {
                break;
            }
        }

        info!("Finished shutting down Youtube Adapter");

        Ok(())
    }

    fn adt_type(&self) -> AdapterType {
        AdapterType::Youtube
    }
}

#[instrument(skip(data))]
async fn download_meta(
    meta: Meta,
    offline_folder: PathBuf,
    mut data: OfflineData,
) -> Result<OfflineData, Error> {
    info!("[Youtube] download_meta: {:?}", meta.url());

    match tokio::task::spawn_blocking(move || download_youtube(&meta, offline_folder)).await? {
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

#[instrument(level = "debug")]
pub fn valid_url(url: Option<&Url>) -> bool {
    if let Some(url) = url {
        if let Some(domain) = url.domain() {
            if (domain.ends_with("youtube.com") || domain.ends_with("youtu.be"))
                && YoutubeAdapter::yt_dl_installed()
            {
                return true;
            }
        }
    }
    false
}

#[instrument(skip(folder))]
fn download_youtube(meta: &Meta, folder: impl AsRef<Path>) -> Result<PathBuf, Error> {
    let filename = meta.id();
    let folder = folder.as_ref().join(meta.id.clone());
    if read_dir(&folder).is_err() && create_dir_all(&folder).is_err() {
        return Err(Error::FileStoreNoPath);
    }
    let file_path = folder.join(filename);
    tracing::info!("[youtube] Download path {:?}", file_path);
    let url_str;
    if let Some(url) = &meta.url {
        url_str = url.as_str();
    } else {
        return Err(Error::Custom("Youtube-dl error".to_string()));
    }

    let mut available_subs = Command::new("youtube-dl");
    available_subs.arg(url_str).arg("--list-subs");
    let out = str::from_utf8(available_subs.output()?.stdout.as_slice())
        .unwrap()
        .to_string();
    let subs_command;
    if out.contains("Available subtitles") {
        subs_command = "--write-sub"
    } else if out.contains("Available automatic captions") {
        subs_command = "--write-auto-sub"
    } else {
        subs_command = "-q";
        info!("Video {:?} does not have subs", meta.name);
    }

    match Command::new("youtube-dl")
        .arg(url_str)
        .arg("--recode-video")
        .arg("mkv")
        .arg(subs_command)
        .arg("-o")
        .arg(&file_path)
        .stdout(Stdio::null())
        .output()
    {
        Ok(_) => Ok(folder.join(meta.id.clone() + ".mkv")),
        _ => Err(Error::Custom("Youtube-dl error".to_string())),
    }
}
