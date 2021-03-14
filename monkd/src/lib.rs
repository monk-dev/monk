pub mod adapter;
pub mod daemon;
pub mod error;
pub mod index;
pub mod metadata;
pub mod server;
pub mod settings;
pub mod status;

use anyhow::Result;

use crate::adapter::{http::HttpAdapter, youtube::YoutubeAdapter, Adapter, AdapterType};
use crate::daemon::Daemon;
use crate::server::{request::Request, response::Response, Server};
use crate::settings::Settings;

use async_channel::Sender;
use async_lock::Lock;
use directories_next::ProjectDirs;
use std::net::SocketAddr;
use tokio::time::timeout;

use std::time::{Duration, Instant};
use tokio::sync::oneshot;

#[tracing::instrument(skip(settings))]
pub async fn run(settings: Settings) -> Result<()> {
    let start_time = Instant::now();

    let (sender, receiver) = async_channel::unbounded();
    let (shutdown, signal) = oneshot::channel::<()>();

    let addr = SocketAddr::new(settings.daemon().address, settings.daemon().port);

    tokio::spawn(Server::spawn(addr, sender.clone(), signal));
    let timeout_duration = Duration::from_millis(settings.daemon().timeout as u64);

    let adapters = create_adapters(&settings, sender.clone());

    let mut daemon = match Daemon::new(&settings, sender.clone(), adapters.clone()) {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("error creating daemon: {}", e);
            std::process::exit(1);
        }
    };

    'main: loop {
        // let mut timeout = tokio::time::delay_for(timeout_duration).boxed().fuse();
        let request_future = timeout(timeout_duration, receiver.recv());

        if let Ok(request) = request_future.await {
            let (request, response) = request?;

            if let Request::Stop = request {
                tracing::info!("Stop Request Received");

                // Only send an `Ok` if a response is requested
                if let Some(response) = response {
                    let _ = response.send(Response::Ok);
                }

                break 'main;
            }

            tracing::trace!("Recieved request: {:?}", request);

            let res = match daemon.handle_request(request).await {
                r @ Ok(_) => r,
                Err(e) if e.is_client_error() => Ok(Response::from(e)),
                Err(e) => Err(e),
            }?;

            tracing::trace!("Processed Result: {:?}", res);
            tracing::trace!("Result requested: {}", response.is_some());

            // Only send the respone if it was requested
            if let Some(response) = response {
                let _ = response.send(res);
            }
        } else {
            tracing::info!("Timeout Reached");
            break;
        }
    }

    // Shutdown all adapters first, making sure
    // all messages are sent. This is weird since
    // we're calling "handle" request on the adapters
    // again. Adapters should still function after a shutdown
    // though only adapter messages will be handled.
    // Basically we just close off outside messages until
    // all adapter messages are finished.
    for adapter in adapters.into_iter() {
        let mut adapter = adapter.lock().await;

        if let Err(e) = adapter.shutdown().await {
            tracing::error!("error shuting adapter down: {}", e);
        }
    }

    tracing::info!("Handling remaing messages");
    tracing::info!("{} adapter messages left", receiver.len());

    while !receiver.is_empty() {
        if let Ok((request, response)) = receiver.recv().await {
            let res = match daemon.handle_request(request).await {
                r @ Ok(_) => r,
                Err(e) if e.is_client_error() => Ok(Response::from(e)),
                Err(e) => Err(e),
            }?;

            // Only send the respone if it was requested
            if let Some(response) = response {
                let _ = response.send(res);
            }
        }
    }

    let _ = shutdown.send(());

    tracing::info!(
        "Server is shutting down after {:3.4} s.",
        start_time.elapsed().as_secs_f32()
    );

    daemon.shutdown().await?;

    Ok(())
}

pub fn generate_id() -> String {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric).to_ascii_lowercase())
        .take(10)
        .collect()
}

pub fn get_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", "monk")
}

fn create_adapters(
    settings: &Settings,
    sender: Sender<(Request, Option<oneshot::Sender<Response>>)>,
) -> Vec<Lock<Box<dyn Adapter>>> {
    let mut adapters: Vec<Lock<Box<dyn Adapter>>> = Vec::new();

    for a_type in settings.adapters() {
        match a_type {
            AdapterType::Youtube => adapters.push(Lock::new(Box::new(YoutubeAdapter::new(
                settings.offline().data_folder.clone(),
                sender.clone(),
            )))),
            AdapterType::Http => adapters.push(Lock::new(Box::new(HttpAdapter::new(
                settings.offline().data_folder.clone(),
                sender.clone(),
            )))),
        }
    }

    adapters
}
