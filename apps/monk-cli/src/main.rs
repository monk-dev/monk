mod args;

use std::{
    fmt::Write,
    path::{Path, PathBuf},
};

use args::Command;
use colored::{Color, Colorize};
use dialoguer::Confirm;
use monk::types::{
    config::MonkConfig, AddItem, CreateLink, DeleteItem, DeleteLink, GetItem, LinkedItems,
    ListItem, MonkTrait, Search,
};
use monk::types::{Item, SearchResult};
use monk::Monk;

use clap::Parser;
use tracing::metadata::LevelFilter;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::util::SubscriberInitExt;

const COLORS: &[Color] = &[Color::Green, Color::Cyan, Color::White, Color::Yellow];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Only use dotenv when developing
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    let args = args::Args::parse();
    if args.verbose {
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

    let (config, config_file, new_install) = ensure_config(args.config)?;
    let mut monk = Monk::from_config(config.clone()).await?;

    if new_install {
        print_new_install_info(&config_file, &config);
    }

    match args.command {
        Command::List => {
            println!("{}", "Listing Items\n".bright_green().bold());
            let items = monk.list(ListItem::default()).await?;

            let output: Result<Vec<String>, _> = items.iter().map(item_display_string).collect();
            let output = output?.join("\n");

            if !output.is_empty() {
                print!("{output}");
            } else {
                println!(
                    "Monk has no items.\n{}{}",
                    "tip: ".bold(),
                    "Run `monk add --help` to learn how to add an item.".italic()
                );
            }
        }
        Command::Get { id, body } => {
            println!("{}{}", "Getting Item: ".bright_green().bold(), id);
            let item = monk.get(GetItem { id }).await?;
            if let Some(item) = item {
                if body {
                    println!("{}", item.body.unwrap());
                } else {
                    println!("{}", item_display_string(&item)?);
                }
            } else {
                println!("{}", "Item not found".red().italic());
            }
        }
        Command::Add {
            name,
            url,
            comment,
            tags,
        } => {
            println!("  {} {:?} to monk", "Adding".bright_green().bold(), name);

            if let Some(ref url) = url {
                println!("\t{}{}", "url: ".bold(), url);
            }

            println!();

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
            println!("  {}{id}", "Opening: ".bright_green().bold());

            let item = monk.get(GetItem { id }).await?;

            if let Some(item) = item {
                println!("  {}{}", "Found Item: ".bright_green().bold(), item.name);

                if let Some(blob) = item.blob {
                    println!("   {}{}", "Found Blob: ".bright_green().bold(), blob.id);
                    println!("    {}", "Opening Blob".bright_green().bold());
                    match open::that(&blob.path) {
                        Ok(_) => println!(
                            "  {}{}",
                            "Successfully opened: ".bright_green().bold(),
                            item.name
                        ),
                        Err(e) => println!("  {}{}\n\t{}", "Unable to open: ".red(), item.name, e),
                    };
                } else {
                    println!("  {}\n", "Item is not downloaded".red());
                    println!(
                        "To download the item, run:\n\n\tmonk download {}\n\n",
                        item.id
                    );
                }
            }
        }
        Command::Search { count, query } => {
            let query = query.join(" ");
            println!(
                "  {}{query}\n",
                "Searching monk with query: ".bright_green().bold()
            );

            let results = monk
                .search(Search {
                    count: Some(count),
                    query,
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
        Command::Config => {
            println!("config file: {}\n", config_file.display());
            print!("{}", serde_yaml::to_string(&config)?);
        }
        Command::Clean { choice } => {
            println!(
                "  {}{:?}, {} this action is {}",
                "Cleaning: ".bright_green().bold(),
                choice,
                "WARNING:".bright_yellow().bold(),
                "NON REVERSIBLE".bright_red().italic()
            );

            if Confirm::new()
                .wait_for_newline(true)
                .default(false)
                .with_prompt(&format!("  Clean {:?}?", choice))
                .interact()?
            {
                println!("  {}{:?}", "Cleaning: ".bright_blue().bold(), choice);
                println!("unimplemented");
            }
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
            "  ({}) ",
            format!("{:2.2}", search_result.score).purple()
        )?;
    }

    if let Some(search_result) = search_result {
        writeln!(
            title,
            "{}",
            highlight_text(&item.name, &search_result.snippets.name.highlighted).underline()
        )?;
    } else {
        writeln!(title, "  {}", item.name.underline())?;
    };

    let mut body = String::new();

    writeln!(body, "\t{}", item.id.to_string().dimmed())?;

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

    // Url
    if let Some(url) = &item.url {
        writeln!(body, "\t{}{url}", "url: ".bold())?;
    } else {
        writeln!(body, "\t{}{}", "url: ".bold(), "no url".italic())?;
    }

    // Comment
    if let Some(comment) = &item.comment {
        let comment = if let Some(search_result) = search_result {
            highlight_text(comment, &search_result.snippets.comment.highlighted).italic()
        } else {
            comment.italic()
        };

        writeln!(body, "\t{}{}", "comment: ".bold(), comment)?;
    }

    // Created at
    writeln!(
        body,
        "\t{}{}",
        "created at: ".bold(),
        item.created_at.format("%x")
    )?;

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
        highlighted_text.push_str(&text[start..end].yellow());
        start_from = end;
    }

    highlighted_text.push_str(&text[start_from..]);
    highlighted_text.trim().replace('\n', " ")
}

fn ensure_config(config_dir: Option<PathBuf>) -> anyhow::Result<(MonkConfig, PathBuf, bool)> {
    let config_file = monk::config_file_path(config_dir.as_ref())?;
    let new_install = !config_file.exists();

    if new_install {
        println!(
            "  No monk data could be found. Monk will be initalized in: {}\n",
            config_file.parent().unwrap().display()
        );
    }

    let config = monk::get_or_create_config(config_dir.as_deref())?;
    Ok((config, config_file, new_install))
}

fn print_new_install_info(config_file: &Path, config: &MonkConfig) {
    println!("  Monk successfully initalized!");
    println!("    config file:\t{}", config_file.display(),);
    println!("    data dir:\t\t{}", config.data_dir.display());
    println!("\tindex:\t\t{}", config.index_path().display());
    println!("\tdownloads:\t{}", config.download_path().display());
    println!("\tstore:\t\t{}\n", config.store_path().display());
}
