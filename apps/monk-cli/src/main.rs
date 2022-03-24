mod args;

use std::{fmt::Write, path::PathBuf};

use args::Command;
use colored::{Color, Colorize};
use monk::types::{
    config::MonkConfig, AddItem, CreateLink, DeleteItem, DeleteLink, GetBlob, GetItem, LinkedItems,
    ListItem, MonkTrait, Search,
};
use monk::types::{Item, SearchResult};
use monk::Monk;

use clap::Parser;
use tracing::metadata::LevelFilter;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::util::SubscriberInitExt;

const COLORS: &'static [Color] = &[Color::Green, Color::Cyan, Color::White, Color::Yellow];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let args = args::Args::parse();
    let config = ensure_config(args.config)?;

    if args.verbose || config.log {
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

            let output: Result<Vec<String>, _> = items.iter().map(item_display_string).collect();
            print!("{}", output?.join("\n"));
        }
        Command::Get { id, body } => {
            let item = monk.get(GetItem { id }).await?;
            if let Some(item) = item {
                if body {
                    println!("{}", item.body.unwrap());
                } else {
                    println!("{}", item_display_string(&item)?);
                }
            } else {
                println!("NOT FOUND");
            }
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
            print!("{}", item_display_string(&item)?);
        }
        Command::Delete { id } => {
            let item = monk.delete(DeleteItem { id }).await?;
            if let Some(item) = item {
                print!("{}", item_display_string(&item)?);
            } else {
                println!("no item was deleted");
            }
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
                    println!("{}", item_display_string(&item)?);
                } else {
                    println!("linked item could not be found. Was the link not deleted?\n\tPlease open an issue on `https://github.com/monk-dev/monk");
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
            print!("Linked {a} to {b}");
        }
        Command::Open { id } => {
            println!("opening: {id}");
            if let Some(blob) = monk.get_blob(GetBlob::ItemId(id.clone())).await? {
                match open::that(&blob.path) {
                    Ok(_) => print!("opened: {blob:?}"),
                    Err(e) => println!("unable to open: {blob:?}, {e}"),
                };
            } else {
                print!("no blob found: {id}");
            }
        }
        Command::Search { count, query } => {
            let results = monk
                .search(Search {
                    count: Some(count),
                    query: query.join(" "),
                })
                .await?;

            let mut result_strings = Vec::new();
            for result in results {
                if let Some(item) = monk
                    .get(GetItem {
                        id: result.id.to_string(),
                    })
                    .await?
                {
                    result_strings.push(item_search_result_string(&item, &result)?);
                }
            }

            print!("{}", result_strings.join("\n"));
        }
    }

    Ok(())
}

fn item_display_string(item: &Item) -> anyhow::Result<String> {
    item_display_string_inner(item, None)
}

fn item_search_result_string(item: &Item, search_result: &SearchResult) -> anyhow::Result<String> {
    item_display_string_inner(item, Some(search_result))
}

fn item_display_string_inner(
    item: &Item,
    search_result: Option<&SearchResult>,
) -> anyhow::Result<String> {
    // Title
    let mut title = String::new();

    if let Some(search_result) = search_result {
        write!(
            title,
            "({}) ",
            format!("{:2.2}", search_result.score).purple()
        )?;
    }

    if let Some(search_result) = search_result {
        write!(
            title,
            "{} ",
            highlight_text(&item.name, &search_result.snippets.name.highlighted).underline()
        )?;
    } else {
        write!(title, "{} ", item.name.underline())?;
    };

    write!(
        title,
        "({}): ",
        item.created_at.format("%x").to_string().green()
    )?;

    // Url
    if let Some(url) = &item.url {
        writeln!(title, "{url}")?;
    } else {
        writeln!(title, "{}", "no url".italic())?;
    }

    let mut body = String::new();

    // Body search result
    if let Some(search_result) = search_result {
        if !search_result.snippets.body.fragment.is_empty() {
            writeln!(
                body,
                "\tresult: \"{}\"\n",
                highlight_text(
                    &search_result.snippets.body.fragment,
                    &search_result.snippets.body.highlighted
                )
            )?;
        }
    }

    // Comment
    if let Some(comment) = &item.comment {
        let comment = if let Some(search_result) = search_result {
            highlight_text(comment, &search_result.snippets.comment.highlighted).italic()
        } else {
            comment.italic()
        };

        writeln!(body, "\t{}", comment)?;
    }

    // Tags
    if !item.tags.is_empty() {
        write!(body, "\t")?;
        let tags: Vec<String> = item
            .tags
            .iter()
            .zip(COLORS.iter().cycle())
            .map(|(tag, color)| format!("{}", tag.tag.color(*color).italic()))
            .collect();

        writeln!(body, "{}", tags.join(", "))?;
    }

    Ok(format!("{}{}", title, body))
}

fn highlight_text(text: &str, ranges: &[(usize, usize)]) -> String {
    // Adapted from tantivy's html highlighting implementation:
    let mut highlighted_text = String::new();
    let mut start_from = 0;

    for (start, end) in ranges.iter().copied() {
        highlighted_text.push_str(&text[start_from..start]);
        highlighted_text.push_str(&text[start..end].yellow().to_string());
        start_from = end;
    }

    highlighted_text.push_str(&text[start_from..]);
    highlighted_text.trim().replace("\n", " ")
}

fn ensure_config(config_dir: Option<PathBuf>) -> anyhow::Result<MonkConfig> {
    let config_file = monk::config_file_path(config_dir.as_ref())?;

    if !config_file.exists() {
        println!(
            "  No Monk config could be found. A new monk config will be \nplaced in: {}",
            config_file.parent().unwrap().display()
        );
    }

    monk::get_or_create_config(config_dir.as_deref())
}
