use std::fs::File;
use std::path::PathBuf;

use monk::types::config::MonkConfig;
use monk::types::{
    AddItem, CreateLink, DeleteItem, DeleteLink, GetBlob, GetItem, LinkedItems, ListItem,
    MonkTrait, Search,
};
use monk::Monk;

use clap::{Parser, Subcommand};
use tracing::metadata::LevelFilter;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long)]
    pub config: Option<PathBuf>,
    #[clap(short, long)]
    pub verbose: bool,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    List,
    Get {
        id: String,
    },
    Add {
        name: Option<String>,
        url: Option<String>,
        comment: Option<String>,
        #[clap(short, long)]
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
        #[clap(short, long, default_value = "5")]
        count: usize,
        /// A properly structured search query
        query: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let config: MonkConfig = if let Some(config_path) = args.config {
        let file = File::open(config_path)?;
        serde_yaml::from_reader(file)?
    } else {
        MonkConfig::default()
    };

    if config.log {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::from_default_env()
                    .add_directive(LevelFilter::INFO.into())
                    .add_directive("sqlx::query=warn".parse()?)
                    .add_directive("html5ever::serialize=off".parse()?)
                    .add_directive("tantivy=warn".parse()?),
            )
            .finish()
            .init();
    }

    let mut monk = Monk::from_config(config).await?;

    match args.command {
        Command::List => {
            let items = monk.list(ListItem::default()).await?;

            for item in items {
                println!("{item:?}");
            }
        }
        Command::Get { id } => {
            let item = monk.get(GetItem { id }).await?;
            println!("{item:?}");
        }
        Command::Add {
            name,
            url,
            comment,
            tags,
        } => {
            let item = monk
                .add(AddItem {
                    name,
                    url,
                    body: None,
                    comment,
                    tags,
                })
                .await?;
            println!("{item:?}");
        }
        Command::Delete { id } => {
            let item = monk.delete(DeleteItem { id }).await?;
            println!("{item:?}");
        }
        Command::LinkedItems { id } => {
            let items = monk.linked_items(LinkedItems { id }).await?;

            for item in items {
                let item = monk
                    .get(GetItem {
                        id: item.to_string(),
                    })
                    .await?;

                if let Some(item) = item {
                    println!("{:?}", item);
                }
            }
        }
        Command::Link { a, b } => {
            monk.link(CreateLink {
                a: a.clone(),
                b: b.clone(),
            })
            .await?;
            println!("Linked {a} to {b}");
        }
        Command::Unlink { a, b } => {
            monk.unlink(DeleteLink {
                a: a.clone(),
                b: b.clone(),
            })
            .await?;
            println!("Linked {a} to {b}");
        }
        Command::Open { id } => {
            println!("opening: {id}");
            if let Some(blob) = monk.get_blob(GetBlob::ItemId(id.clone())).await? {
                match open::that(&blob.path) {
                    Ok(_) => println!("opened: {blob:?}"),
                    Err(e) => println!("unable to open: {blob:?}, {e}"),
                };
            } else {
                println!("no blob found: {id}");
            }
        }
        Command::Search { count, query } => {
            let results = monk
                .search(Search {
                    count: Some(count),
                    query: query.join(" "),
                })
                .await?;

            for result in results {
                println!("{result:?}");
            }
        }
    }

    Ok(())
}
