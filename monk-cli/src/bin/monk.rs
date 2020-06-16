use config::{Config, Environment, File};
use structopt::StructOpt;

use monk_cli::{args::Args, cli::Cli, error::Error, args::Subcommand};
use monkd::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // println!("{}", std::env::current_dir().unwrap().display());

    let args = Args::from_args();

    // let mut config = Config::default();
    // config.merge(File::with_name(&args.config.to_str().unwrap()))?;
    // config.merge(Environment::with_prefix("sack_cli"))?;

    // let settings: Settings = config.try_into()?;

    if args.subcommand == Subcommand::DefaultConfig {
        println!("{}", serde_yaml::to_string(&Settings::default()).unwrap());
        std::process::exit(0);
    }

    let settings = Settings::get_settings(args.config.clone()).unwrap();

    if let Err(e) = Cli::run(settings, args).await {
        println!("error: {}", e);
    }

    Ok(())
}
