#![feature(result_flattening, with_options)]

pub mod adapter;
pub mod daemon;
pub mod error;
pub mod index;
pub mod metadata;
pub mod server;
pub mod settings;

use anyhow::Result;

use crate::daemon::Daemon;
use crate::server::{request::Request, response::Response, Server};
use crate::settings::Settings;
use crate::adapter::{http::HttpAdapter, AdapterSlug, Adapter};

use directories_next::ProjectDirs;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::time::delay_for;
use futures::{StreamExt, FutureExt};
use futures::select;
use async_channel::{Sender, Receiver};
use async_lock::Lock;

#[tracing::instrument(skip(settings))]
pub async fn run(settings: Settings) -> Result<()> {

    let start_time = Instant::now();

    let (sender, mut http_receiver) = mpsc::channel(100);
    let (adapter_sender, mut adapter_receiver) = async_channel::bounded(100);
    let (shutdown, signal) = oneshot::channel::<()>();

    let addr = SocketAddr::new(settings.daemon().address, settings.daemon().port);

    tokio::spawn(Server::spawn(addr, sender, signal));
    let timeout_duration = Duration::from_millis(settings.daemon().timeout as u64);

    let adapters = create_adapters(&settings, adapter_sender);

    let mut daemon = match Daemon::new(&settings, adapters.clone()) {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("error creating daemon: {}", e);
            std::process::exit(1);
        }
    };
 
    
    'main: loop {
        let mut timeout = tokio::time::delay_for(timeout_duration).boxed().fuse();
        let request = select! {
            a = http_receiver.next().fuse() => {
                Some(a)
            },
            a = adapter_receiver.next().fuse() => {
                Some(a)
            },
            t = timeout => {
                None
            }
        };

        if let Some(Some((request, response))) = request {
            if let Request::Stop = request {
                tracing::info!("Stop Request Received");
                
                // Only send an `Ok` if a response is requested
                if let Some(response) = response {
                    let _ = response.send(Response::Ok);
                }

                break 'main;
            }

            let res = match daemon.handle_request(request).await {
                r @ Ok(_) => r,
                Err(e) if e.is_client_error() => Ok(Response::from(e)),
                Err(e) => Err(e),
            }?;

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
    tracing::info!("{} adapter messages left", adapter_receiver.len());

    while !adapter_receiver.is_empty() {
        if let Ok((request, response)) = adapter_receiver.recv().await {
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
    ProjectDirs::from("io", "darling", "monk")
}

fn create_adapters(settings: &Settings, sender: Sender<(Request, Option<oneshot::Sender<Response>>)>) -> Vec<Lock<Box<dyn Adapter>>> {
    let mut adapters: Vec<Lock<Box<dyn Adapter>>> = Vec::new();

    for slug in settings.adapters() {
        match slug {
            AdapterSlug::Http => adapters.push(Lock::new(Box::new(HttpAdapter::new(settings.offline().path.clone(), sender.clone())))),
        }
    }

    adapters
}