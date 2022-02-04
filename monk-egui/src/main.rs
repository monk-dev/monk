use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use monk::types::config::MonkConfig;
use monk_egui::{MonkApp, MonkServer};
use tracing::metadata::LevelFilter;
use tracing::{error, info};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(LevelFilter::INFO.into())
                .add_directive("sqlx::query=warn".parse()?)
                .add_directive("html5ever::serialize=off".parse()?)
                .add_directive("tantivy=warn".parse()?),
        )
        .finish()
        .init();

    let config = MonkConfig::default();
    let (tx_client, rx_server) = crossbeam_channel::bounded(1);
    let (tx_server, rx_client) = crossbeam_channel::bounded(1);
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = Arc::clone(&cancel);

    std::thread::spawn(|| {
        info!("creating runtime");
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                error!(%error, "error creating runtime");
                cancel_clone.store(true, Ordering::Relaxed);
                return;
            }
        };

        info!("blocking on monk_server");
        runtime.block_on(async move {
            let monk_server = match MonkServer::from_config(config, tx_server, rx_server).await {
                Ok(server) => server,
                Err(error) => {
                    error!(%error, "server error");
                    cancel_clone.store(true, Ordering::Relaxed);
                    return;
                }
            };

            info!("running monk_server");
            monk_server.run(cancel_clone).await;
        });
    });

    let mut app = MonkApp::init(cancel, tx_client, rx_client)?;
    app.load_items()?;

    info!("starting app");
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
