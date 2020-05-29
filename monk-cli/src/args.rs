use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
pub struct Args {
    pub config: Option<PathBuf>,
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
pub enum Subcommand {
    /// Print the the final monk config with optional config path.
    ///
    /// The outputted config is equivalent to the config created
    /// from `monkd -c [file]`
    Config {
        file: Option<PathBuf>,
    },
    /// Prints the default config for monkd
    DefaultConfig,
    /// Add an item to the database
    Add {
        /// Optional name of an item
        name: Option<String>,
        /// The uri of the item
        #[structopt(short, long)]
        url: Option<String>,
        #[structopt(short, long)]
        comment: Option<String>,
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
        /// How many items to return. Defaults to all
        // #[structopt(short, long)]
        count: Option<usize>,
    },
    /// Get a single item from the database
    Get {
        id: String,
    },
    /// Delete an item from the database
    Delete {
        id: String,
    },
    /// Download either a single ID or all ids
    Download {
        // #[structopt(short, long)]
        // all: bool,
        id: String,
    },
    Open {
        id: String,
    },
    /// Shutdown the daemon with no cleanup
    ForceShutdown,
    /// Cleanly shutdown the daemon
    Stop,
}
