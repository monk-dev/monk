#![feature(result_flattening, with_options)]

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

use directories_next::ProjectDirs;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

#[tracing::instrument(skip(settings))]
pub async fn run(settings: Settings) -> Result<()> {
    use tokio::stream::StreamExt;

    let start_time = Instant::now();

    let (sender, mut receiver) = mpsc::channel(100);
    let (shutdown, signal) = oneshot::channel::<()>();

    let addr = SocketAddr::new(settings.daemon().address, settings.daemon().port);

    tokio::spawn(Server::spawn(addr, sender, signal));
    let timeout_duration = Duration::from_millis(settings.daemon().timeout as u64);

    let mut daemon = match Daemon::new(&settings) {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("error creating daemon: {}", e);
            std::process::exit(1);
        }
    };

    'main: loop {
        let request = timeout(timeout_duration, receiver.next()).await;

        if let Ok(Some((request, response))) = request {
            if let Request::Stop = request {
                tracing::info!("Stop Request Received");

                let _ = response.send(Response::Ok);
                let _ = shutdown.send(());
                break 'main;
            }

            let res = match daemon.handle_request(request).await {
                r @ Ok(_) => r,
                Err(e) if e.is_client_error() => Ok(Response::from(e)),
                Err(e) => Err(e),
            }?;

            let _ = response.send(res);
        } else if let Ok(None) = request {
            continue;
        } else {
            tracing::info!("Timeout Reached");

            let _ = shutdown.send(());
            break;
        }
    }

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
