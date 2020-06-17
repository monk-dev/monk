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
    /// Download either a single ID or all ids if empty
    Download {
        // #[structopt(short, long)]
        // all: bool,
        id: Option<String>,
    },
    Open {
        id: String,
    },
    /// Run an ID through the full text search indexing pipeline
    Index {
        id: String,
    },
    /// Search for metadata based off of the given query
    ///
    /// The query grammar is very simplistic. A query is tokenized and an
    /// "OR" is inserted between tokens. Remove the '`' when writing a query
    /// on the CLI. Quotes are used for phrase queries. {n}{n}
    /// 1. `sea whale` for results containing "sea" OR "whale" {n}
    /// 2. `+sea -whale` for results that must have "sea" and not have "whale" {n}
    /// 3. `pears AND apples` for a conjunction of the two {n}
    /// 4. `"Phrase Query"` for "phrase" followed by "query" (use quotes). {n}
    /// 5. `*` for simply everything.
    ///
    /// The query grammar can be found here: https://docs.rs/tantivy/0.12.0/tantivy/query/struct.QueryParser.html
    Search {
        /// A properly structured search query
        query: Vec<String>,
    },
    /// Shutdown the daemon with no cleanup
    ForceShutdown,
    /// Cleanly shutdown the daemon
    Stop,
}
