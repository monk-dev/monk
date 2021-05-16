use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
pub struct Args {
    #[structopt(skip)]
    pub oneline: bool,
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
    Config { file: Option<PathBuf> },
    /// Prints the default config for monkd
    DefaultConfig,
    /// Add an item to the database
    Add {
        /// Optional name of an item
        name: Option<String>,
        /// The url of the item
        #[structopt(short, long)]
        url: Option<String>,
        /// Any associated comment for the item
        #[structopt(short, long)]
        comment: Option<String>,
        /// A space seperated list of tags for an article
        #[structopt(short, long)]
        tags: Vec<String>,
    },
    /// List all items in the database
    List {
        /// Print lists of metadata items on a single line. Much like `git log --oneline`
        #[structopt(short, long)]
        oneline: bool,
        /// Limit how many items are returned. Defaults to all items
        #[structopt(short, long)]
        count: Option<usize>,
        /// List only articles with specified tags
        tags: Vec<String>,
    },
    /// Get a single item from the database
    Get { id: String },
    /// Edit the name, url, or comment of a single item from the database. Fields are optional.
    Edit {
        /// The name or id of the item you wish to edit
        id: String,
        /// New Name
        #[structopt(short, long)]
        name: Option<String>,
        #[structopt(short, long)]
        url: Option<String>,
        #[structopt(short, long)]
        comment: Option<String>,
        #[structopt(short, long)]
        add_tags: Vec<String>,
        #[structopt(short, long)]
        remove_tags: Vec<String>,
    },
    /// Delete an item from the database
    Delete { id: String },
    /// Download either a single ID or all ids if empty
    Download {
        // #[structopt(short, long)]
        // all: bool,
        id: Option<String>,
    },
    /// Open an ID with the system's default program for the item's filetype.
    Open {
        /// Block until the item is downloaded and ready to be opened
        #[structopt(short, long)]
        blocking: bool,
        /// Open the url instead of the offline store.
        #[structopt(short, long)]
        online: bool,
        /// The ID of the item to open
        id: String,
    },
    /// Run an ID through the full text search indexing pipeline. Simply write
    /// an ID after `index` to index that ID.
    ///
    /// Example: `monk index t4v`
    Index {
        // /// Retrieve the indexing status for the given ID
        // #[structopt(short, long)]
        // status: bool,
        /// Indexing subcommand. Write an ID to simply index that ID.
        #[structopt(subcommand)]
        command: IndexSubcommand,
    },
    /// Import a store.json file.
    Import { file: String },
    /// Export monks store.
    Export {
        file: String,
        /// Export all articles in a tar file.
        #[structopt(short, long)]
        full: bool,
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
        /// Print lists of metadata items on a single line. Much like `git log --oneline`
        #[structopt(short, long)]
        oneline: bool,
        /// Maximum number of items to return
        #[structopt(short, long, default_value = "5")]
        count: usize,
        /// A properly structured search query
        query: Vec<String>,
    },
    /// Get the status of various parts of the daemon. Simply write
    /// an ID rather than a subcommand to get the status of that ID.
    Status {
        /// The type of status to retrieve.
        #[structopt(subcommand)]
        kind: StatusRequestKind,
    },
    /// Shutdown the daemon with no cleanup
    ForceShutdown,
    /// Cleanly shutdown the daemon
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, StructOpt)]
pub enum IndexSubcommand {
    /// Get the current index status of an ID
    Status { id: String },
    /// Index everything
    All {
        /// Only index articles with matching tags
        #[structopt(short, long)]
        tags: Vec<String>,
    },
    /// Index the given ID
    #[structopt(external_subcommand)]
    Id(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, StructOpt)]
pub enum StatusRequestKind {
    /// Get the status of the meta store, offline store, and search index.
    All,
    /// Get the status of the search index.
    Index,
    /// Get the status of meta store.
    Store,
    /// Get the status of offline store.
    Offline,
    #[structopt(external_subcommand)]
    Id(Vec<String>),
}
