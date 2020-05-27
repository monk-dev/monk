use config::{Config, Environment, File};
use structopt::StructOpt;

use monk_cli::{args::Args, cli::Cli, error::Error};
use monkd::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // println!("{}", std::env::current_dir().unwrap().display());

    let args = Args::from_args();

    let mut config = Config::default();
    config.merge(File::with_name(&args.config.to_str().unwrap()))?;
    config.merge(Environment::with_prefix("sack_cli"))?;

    let settings: Settings = config.try_into()?;

    Cli::run(settings, args).await?;

    Ok(())
}
