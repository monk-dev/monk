use colored::*;
use scraper::{Html, Selector};
use std::net::SocketAddr;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};
use url::Url;

use crate::args::{Args, IndexSubcommand, StatusRequestKind, Subcommand};
use crate::error::Error;

use monkd::metadata::Meta;
use monkd::server::{
    request::{Edit, Request, StatusKind},
    response::{Response, SnippetDef},
};
use monkd::settings::Settings;
use monkd::status::StatusResponse;

pub struct Cli;

impl Cli {
    pub async fn run(settings: Settings, mut args: Args) -> Result<(), Error> {
        check_or_spawn(&settings).await?;

        let request = match args.subcommand.clone() {
            Subcommand::Config { file } => {
                let config = Settings::get_settings(file).unwrap();
                println!("{}", serde_yaml::to_string(&config).unwrap());
                std::process::exit(0);
            }
            Subcommand::DefaultConfig => {
                let settings = Settings::default();
                println!("{}", serde_yaml::to_string(&settings).unwrap());
                std::process::exit(0);
            }
            Subcommand::Add {
                mut name,
                url,
                comment,
            } => {
                if name.is_none() && url.is_none() && comment.is_none() {
                    println!("either name, url, or comment must be set");
                    std::process::exit(1);
                }
                if let Some(ref u) = url {
                    let res = reqwest::get(u).await?;
                    let body = res.text().await?;
                    if name.is_none() {
                        let fragment = Html::parse_fragment(&body); // Parse the top of the page
                        let selector = Selector::parse("title").unwrap();
                        if let Some(title) = fragment.select(&selector).next() {
                            name = Some(title.inner_html());
                        }
                    }
                    // TODO: Scrape for a description
                }

                let url: Option<Url> = url.map(|s| Url::parse(&s)).transpose()?;

                Request::Add { name, url, comment }
            }
            Subcommand::List { oneline, count } => {
                args.oneline = oneline;
                Request::List { count }
            }
            Subcommand::Edit {
                id,
                name,
                url,
                comment,
            } => Request::Edit {
                id,
                edit: Edit {
                    name: name,
                    url: url,
                    comment: comment,
                },
            },
            Subcommand::Delete { id } => Request::Delete { id },
            Subcommand::Get { id } => Request::Get { id },
            Subcommand::Search {
                oneline,
                count,
                query,
            } => {
                args.oneline |= oneline;
                let query = query.join(" ");

                Request::Search {
                    count: Some(count),
                    query,
                }
            }
            Subcommand::Index { command } => match command {
                IndexSubcommand::Status { id } => Request::IndexStatus { id },
                IndexSubcommand::Id(id) => Request::Index { id: id[0].clone() },
                IndexSubcommand::All => Request::IndexAll,
            },
            Subcommand::Status { kind } => match kind {
                StatusRequestKind::All => Request::Status {
                    kind: StatusKind::All,
                },
                StatusRequestKind::Index => Request::Status {
                    kind: StatusKind::Index,
                },
                StatusRequestKind::Offline => Request::Status {
                    kind: StatusKind::Offline,
                },
                StatusRequestKind::Store => Request::Status {
                    kind: StatusKind::Store,
                },
                StatusRequestKind::Id(ids) => Request::Status {
                    kind: StatusKind::Id(ids[0].clone()),
                },
            },
            Subcommand::Stop => Request::Stop,
            Subcommand::ForceShutdown => Request::ForceShutdown,
            Subcommand::Download { id } => Request::Download { id },
            Subcommand::Open { id, online, .. } => Request::Open { id, online },
        };

        let socket = SocketAddr::new(settings.daemon().address, settings.daemon().port);
        let socket_url = format!("http://{}", socket.to_string());
        let url = reqwest::Url::parse(&socket_url)?;

        let response: Response = reqwest::Client::new()
            .get(url.clone())
            .json(&request)
            .send()
            .await?
            .json::<Response>()
            .await?;

        handle_response(&args, response);

        Ok(())
    }
}

pub fn handle_response(args: &Args, response: Response) {
    match response {
        Response::Item(meta) => {
            print_tabled(vec![meta]);
        }
        Response::List(mut items) => {
            items.sort_by_key(|i| *i.found());

            if items.is_empty() {
                println!("monk's store is empty");
                println!(
                    "use {} to add a new item",
                    "monk add [-n <name>] -u <url>".yellow(),
                )
            } else {
                if args.oneline {
                    print_oneline(items);
                } else {
                    print_tabled(items);
                }
            }
        }
        Response::TooManyMeta(id, metas) => print_too_many(id, metas),
        Response::NewId(id) => {
            println!("[{}] created", id.to_string().bright_purple());
        }
        Response::NotFound(id) => {
            println!("[{}] not found", id.to_string().bright_purple());
            println!(
                "{} to list all items and IDs in the store",
                "monk list".yellow()
            );
        }
        Response::NoAdapterFound(id) => {
            println!(
                "An adapter that could handle [{}] could not be found.",
                id.to_string().bright_purple()
            );
        }
        Response::MetaOfflineStatus(id, status) => {
            println!(
                "status for [{}]: {:?}",
                id.to_string().bright_purple(),
                status
            );
        }
        Response::IndexStatus(id, status) => {
            println!(
                "[{}] {}",
                id.to_string().bright_purple(),
                status
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_else(|| "not indexed".into())
            );
        }
        Response::Indexing(id) => {
            println!("[{}] indexing", id.to_string().bright_purple());
        }
        Response::OpenStatus(id, status) => {
            println!(
                "[{}] cannot be opened: {:?}",
                id.to_string().bright_purple(),
                status
            );
        }
        Response::Open(path) => {
            open::that(path).unwrap();
        }
        Response::Unhandled => {
            println!("monk could not handle the request");
        }
        Response::Many(responses) => {
            for response in responses {
                handle_response(args, response);
            }
        }
        Response::Status(status) => {
            print_status(status);
        }
        Response::SearchResult(mut items) => {
            items.sort_by_key(|i| *i.0.found());

            if items.is_empty() {
                println!("No matches found");
            } else {
                print_search(items)
            }
        }
        Response::Custom(string) => {
            println!("{}", string);
        }
        Response::Ok => {}
        Response::Error(e) => {
            println!("error: {}", e);
            // std::process::exit(1);
        }
    }
}

pub fn print_tabled(metas: Vec<Meta>) {
    let table = create_meta_table(metas);
    print!("{}", table.render());
}

pub fn print_too_many(id: String, possible: Vec<Meta>) {
    let mut error = Vec::new();

    error.push(TableCell::new_with_alignment(
        format!("too many meta results for id: `{}`", id),
        1,
        Alignment::Left,
    ));

    let mut error_table = Table::new();
    error_table.max_column_width = 40;
    error_table.style = TableStyle::rounded();

    error_table.add_row(Row::new(error));

    let meta_table = create_meta_table(possible);

    print!("{}", error_table.render());
    print!("{}", meta_table.render());
}

pub fn create_meta_table<'a>(metas: Vec<Meta>) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 40;
    table.style = TableStyle::rounded();

    let mut row = Vec::new();
    row.push(TableCell::new_with_alignment("name", 1, Alignment::Center));
    row.push(TableCell::new_with_alignment("url", 1, Alignment::Center));
    row.push(TableCell::new_with_alignment(
        "comment",
        1,
        Alignment::Center,
    ));
    row.push(TableCell::new_with_alignment("date", 1, Alignment::Center));
    row.push(TableCell::new_with_alignment("id", 1, Alignment::Center));
    // row.push(TableCell::new_with_alignment(
    //     "last opened",
    //     1,
    //     Alignment::Center,
    // ));
    table.add_row(Row::new(row));

    for meta in metas {
        let mut row = Vec::new();
        row.push(TableCell::new_with_alignment(
            meta.name().unwrap_or(""),
            1,
            Alignment::Left,
        ));
        row.push(TableCell::new_with_alignment(
            meta.url().map(|u| u.to_string()).unwrap_or("".into()),
            1,
            Alignment::Left,
        ));

        if let Some(comment) = meta.comment() {
            row.push(TableCell::new_with_alignment(comment, 1, Alignment::Left));
        } else {
            row.push(TableCell::new_with_alignment("n/a", 1, Alignment::Center));
        }

        row.push(TableCell::new_with_alignment(
            meta.found().format("%b %d, %Y").to_string(),
            1,
            Alignment::Center,
        ));

        row.push(TableCell::new_with_alignment(
            meta.id(),
            1,
            Alignment::Right,
        ));

        table.add_row(Row::new(row));
    }

    table
}

fn print_search(results: Vec<(Meta, SnippetDef)>) {
    for (meta, snippet) in results {
        print!("[{}]", meta.id().bright_purple());
        if let Some(name) = meta.name() {
            print!(" {}:", name.yellow());
        }

        if let Some(url) = meta.url() {
            print!(" {}", url.to_string().underline().bright_blue());
        }
        println!();
        // If the snippet length is 0, that mean the match was on the comment or title
        // of the artical. This is because Tantivy does not store comments in
        // the index, and cannot create a snippet.
        if snippet.fragment().len() == 0 {
            match meta.comment() {
                Some(s) => print!("{}", s.blue()),
                _ => (),
            }
            return;
        }
        let mut start_from = 0;
        for (start, end) in snippet.highlighted().iter().map(|h| h.bounds()) {
            print!(
                "{}",
                &snippet.fragment()[start_from..start]
                    .replace("\n", " ")
                    .replace("\r", "")
            );
            print!(
                "{}",
                &snippet.fragment()[start..end]
                    .replace("\n", " ")
                    .replace("\r", "")
                    .bold()
                    .red()
            );
            start_from = end;
        }
        print!(
            "{}",
            &snippet.fragment()[start_from..]
                .replace("\n", " ")
                .replace("\r", "")
        );
        println!();
        println!();
    }
}

fn print_oneline(metas: Vec<Meta>) {
    for meta in metas {
        print!("[{}]", meta.id().bright_purple());
        if let Some(name) = meta.name() {
            print!(" {}:", name.yellow());
        }

        if let Some(url) = meta.url() {
            print!(" {}", url.to_string().underline().bright_blue());
        }

        if let Some(comment) = meta.comment() {
            print!(" {}", comment);
        }

        print!(" {}", meta.found().format("%b %d, %Y").to_string().green());
        println!();
    }
}

fn print_status(status: StatusResponse) {
    if let Some(file_store) = status.file_store {
        println!("{} [{}]:", "File Store".bold(), file_store.version.yellow());
        println!(
            "  {}",
            get_byte_unit(file_store.bytes_on_disk).to_string().green()
        );
        println!("  {} {}", file_store.item_count, "item(s)".blue());

        if status.offline_store.is_some() || status.index_status.is_some() {
            println!();
        }
    }

    if let Some(offline_store) = status.offline_store {
        println!("{}:", "Offline Store".bold());
        println!(
            "  {}",
            get_byte_unit(offline_store.bytes_on_disk)
                .to_string()
                .green()
        );
        println!("  {} {}", offline_store.item_count, "item(s)".blue());

        if status.index_status.is_some() {
            println!();
        }
    }

    if let Some(index) = status.index_status {
        println!("{}:", "Search Index".bold());
        println!(
            "  {}",
            get_byte_unit(index.bytes_on_disk).to_string().green()
        );
        println!("  {} {}", index.item_count, "item(s)".blue());
    }

    if let Some(meta) = status.meta {
        println!("[{}]:", meta.id.bright_purple());
        println!(
            "size:     {}",
            get_byte_unit(meta.bytes_on_disk).to_string().green()
        );
        println!(
            "index:    {}",
            meta.index_status
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| "not indexed".to_string())
        );
        println!(
            "offline:  {}",
            meta.offline_status
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| "not downloaded".to_string())
        );
    }
}

fn get_byte_unit(bytes: usize) -> byte_unit::AdjustedByte {
    let byte = byte_unit::Byte::from_bytes(bytes as u128);
    let adjusted = byte.get_appropriate_unit(false);

    adjusted
}

pub async fn check_or_spawn(settings: &Settings) -> Result<(), std::io::Error> {
    use std::process::{Command, Stdio};
    use sysinfo::{ProcessExt, SystemExt};
    use tokio::net::TcpStream;

    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    if let None = system
        .get_processes()
        .iter()
        .find(|(_pid, proc)| proc.name() == "monkd")
    {
        // println!("Spawning the daemon");

        let command_path = if std::env::var("MONK_DEBUG").is_ok() {
            "./target/debug/monkd"
        } else {
            "monkd"
        };

        let _monkd = Command::new(command_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| {
                println!("error spawning monkd");
                e
            })?;

        tokio::time::delay_for(std::time::Duration::from_millis(100)).await;

        let mut connect_flag = false;
        for _ in 0..5u8 {
            match TcpStream::connect((settings.daemon().address, settings.daemon().port)).await {
                Ok(_) => {
                    connect_flag = true;
                    break;
                }
                Err(_) => {
                    tokio::time::delay_for(std::time::Duration::from_millis(50)).await;
                }
            }
        }

        if !connect_flag {
            println!("error: could not connect to daemon within 350 ms. of spawning");
            std::process::exit(1)
        }
    }

    Ok(())
}
