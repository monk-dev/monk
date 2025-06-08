use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    /// The config folder that Monk loads from.
    #[clap(short, long)]
    pub config: Option<PathBuf>,
    /// Print internal tracing logs to stderr.
    #[clap(short, long)]
    pub verbose: bool,
    /// Output machine-readable newline delimited JSON. Subcommands that return multiple items (list, search, etc) will
    /// output newline-delimited JSON.
    #[clap(short, long)]
    pub json: bool,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List all of the items that are stored in monk.
    List,
    /// Get a single item or its body by the item id.
    Get {
        id: String,
        #[clap(short, long)]
        body: bool,
    },
    /// Add an Item to Monk.
    ///
    /// All Items in Monk must have a `Name`. The url, comment
    /// and tags are optional.
    ///
    /// # Examples
    ///
    /// monk add "My cool article" https://example.com
    ///
    /// monk add "This article has tags" https://article.com -t kernel networking rust
    Add {
        /// The name of the item
        name: String,
        /// The url or local file of the item. Path-based items are
        /// considered "not managed" and will not be deleted nor exported by monk.
        url: Option<String>,
        #[clap(short, long)]
        comment: Option<String>,
        /// The item's tags. Tags let you put items into
        /// different categories.
        ///
        /// # Examples
        ///
        /// monk add "This article has tags" https://article.com -t kernel -t networking -t srust
        #[clap(short, long)]
        tags: Vec<String>,
    },
    /// Delete an item by ID from Monk's store. This will remove the item
    /// from the index and delete any managed (downloaded) items.
    Delete { id: String },
    /// TODO: Convert into link subcommand
    LinkedItems { id: String },
    /// TODO: Convert into link subcommand
    Link { a: String, b: String },
    /// TODO: Convert into link subcommand
    Unlink { a: String, b: String },
    /// Opens the item using whatever default program is set
    /// to open a file with that extension.
    Open { id: String },
    /// Search for an item given a tantivy query string.
    ///
    /// Tantivy's query language is defined in the docs here: https://docs.rs/tantivy/latest/tantivy/query/struct.QueryParser.html
    ///
    /// # Examples
    ///
    /// monk search hello world
    /// monk search title:"Hello World" OR food
    /// monk search networking OR tag:/kernel
    Search {
        #[clap(short, long, default_value = "1")]
        count: usize,
        /// A properly structured search query
        query: Vec<String>,
    },
    /// Prints the config that monk is running with.
    Config,
    /// Clean all or parts of monk's internal data storage.
    Clean {
        #[clap(subcommand)]
        choice: CleanSubcommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum CleanSubcommand {
    All,
    Store,
    Index,
    Downloads,
}
