use sack::{error::Error, Args};
use structopt::StructOpt;

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    sack::run(args)
}
