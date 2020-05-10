use cli::{error::Error, Args};
use structopt::StructOpt;

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    cli::run(args)
}
