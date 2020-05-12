use std::path::PathBuf;

use anyhow::Result;
use config::{Config, Environment, File};
use structopt::StructOpt;

use daemon::settings::Settings;

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
pub struct Args {
    #[structopt(short, long, default_value = "./daemon.yaml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::from_args();

    let mut config = Config::default();
    config.merge(File::with_name(&args.config))?;
    config.merge(Environment::with_prefix("daemon"))?;

    let settings: Settings = config.try_into()?;

    tracing::info!("{:?}", settings);

    daemon::run(settings).await?;

    Ok(())
}
