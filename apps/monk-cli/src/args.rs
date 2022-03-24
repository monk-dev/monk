use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    pub config: Option<PathBuf>,
    #[clap(short, long)]
    pub verbose: bool,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    List,
    Get {
        id: String,
        #[clap(short, long)]
        body: bool,
    },
    Add {
        name: String,
        url: Option<String>,
        comment: Option<String>,
        #[clap(short, long, multiple_values = true)]
        tags: Vec<String>,
    },
    Delete {
        id: String,
    },
    LinkedItems {
        id: String,
    },
    Link {
        a: String,
        b: String,
    },
    Unlink {
        a: String,
        b: String,
    },
    Open {
        id: String,
    },
    Search {
        #[clap(short, long, default_value = "1")]
        count: usize,
        /// A properly structured search query
        query: Vec<String>,
    },
}
