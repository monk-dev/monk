use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing_subscriber::EnvFilter;

use monkd::settings::Settings;

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
pub struct Args {
    #[structopt(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    let settings: Settings = Settings::get_settings(args.config)?;

    let appender = tracing_appender::rolling::daily(settings.log_dir(), "monkd");
    let (non_blocking, _guard) = tracing_appender::non_blocking(appender);
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_env_filter(env_filter)
        .with_writer(non_blocking)
        .init();

    tracing::info!("Starting up");
    tracing::info!("{:?}", settings);

    match monkd::run(settings).await {
        Ok(_) => (),
        Err(e) => tracing::error!("monkd shutting down with fatal error: {:?}", e),
    }

    Ok(())
}
