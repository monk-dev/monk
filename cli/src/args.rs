use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
pub struct Args {
    #[structopt(default_value = "./cli/cli.yaml")]
    pub config: PathBuf,
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
pub enum Subcommand {
    /// Add an item to the database
    Add {
        /// The name of the item
        // #[structopt()]
        name: String,
        /// The uri of the item
        #[structopt(short, long)]
        url: Option<String>,
        // /// The body of the item
        // #[structopt(short, long)]
        // body: Option<String>,
        // /// The type of item: article, project, newsletter, forum, repo
        // #[structopt(name = "type", short, long)]
        // ty: Option<String>,
        // /// Any associated comment for the item
        // #[structopt(short, long)]
        // comment: Option<String>,
    },
    /// List all items in the database
    List {
        // #[structopt(short, long)]
        count: Option<usize>,
    },
    Get {
        id: String,
    },
    Delete {
        id: String,
    },
    ForceShutdown,
    Stop,
}
