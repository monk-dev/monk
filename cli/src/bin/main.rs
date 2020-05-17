use config::{Config, Environment, File};
use structopt::StructOpt;

use cli::settings::Settings;
use cli::{args::Args, cli::Cli, error::Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::from_args();

    let mut config = Config::default();
    config.merge(File::with_name(&args.config.to_str().unwrap()))?;
    config.merge(Environment::with_prefix("sack_cli"))?;

    let settings: Settings = config.try_into()?;

    Cli::run(settings, args).await?;

    Ok(())
}
