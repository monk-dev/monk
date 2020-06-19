use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

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
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(non_blocking)
        .init();

    tracing::info!("Starting up");
    tracing::info!("{:?}", settings);

    monkd::run(settings).await?;

    Ok(())
}
