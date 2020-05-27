use std::path::PathBuf;

use anyhow::Result;
use config::{Config, Environment, File};
use structopt::StructOpt;

use monkd::settings::Settings;

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
pub struct Args {
    #[structopt(short, long, default_value = "./monkd.yaml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {    
    let args = Args::from_args();
    
    let mut config = Config::default();
    config.merge(File::with_name(&args.config))?;
    config.merge(Environment::with_prefix("monk"))?;
    
    let settings: Settings = config.try_into()?;

    let appender = tracing_appender::rolling::daily(settings.log_dir(), "monkd");
    let (non_blocking, _guard) = tracing_appender::non_blocking(appender);
    tracing_subscriber::fmt().with_ansi(false).with_writer(non_blocking).init();

    tracing::info!("{:?}", settings);

    monkd::run(settings).await?;

    Ok(())
}
