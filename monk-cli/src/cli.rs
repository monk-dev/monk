use std::net::SocketAddr;

use crate::args::{Args, Subcommand};
use crate::error::Error;

use monkd::metadata::Meta;
use monkd::server::{request::Request, response::Response};
use monkd::settings::Settings;
use url::Url;

pub struct Cli;

impl Cli {
    pub async fn run(settings: Settings, args: Args) -> Result<(), Error> {
        check_or_spawn().await?;

        let request = match args.subcommand {
            Subcommand::Add { name, url, comment } => {
                let url: Option<Url> = url.map(|s| Url::parse(&s)).transpose()?;

                Request::Add { name, url, comment }
            }
            Subcommand::List { count } => Request::List { count },
            Subcommand::Delete { id } => Request::Delete { id },
            Subcommand::Get { id } => Request::Get { id },
            Subcommand::Stop => Request::Stop,
            Subcommand::ForceShutdown => Request::ForceShutdown,
            Subcommand::Download { id } => Request::Download { id },
            Subcommand::Open { id } => Request::Open { id },
        };

        let socket = SocketAddr::new(settings.daemon().address, settings.daemon().port);
        let socket_url = format!("http://{}", socket.to_string());
        let url = reqwest::Url::parse(&socket_url)?;

        let response: Response = reqwest::Client::new()
            .get(url)
            .json(&request)
            .send()
            .await?
            .json::<Response>()
            .await?;

        match response {
            Response::Item(meta) => {
                print_tabled(vec![meta]);
            }
            Response::List(mut items) => {
                items.sort_by_key(|i| *i.found());

                if items.is_empty() {
                    println!("no items returned");
                } else {
                    print_tabled(items);
                }
            }
            Response::NewId(id) => {
                println!("Created ID: {}", id);
            }
            Response::NotFound(id) => {
                println!("ID not found: {}", id);
            }
            Response::Status(id, status) => {
                println!("Status for `{}`: {:?}", id, status);
            }
            Response::OpenStatus(id, status) => {
                println!("`{}` cannot be open, status: {:?}", id, status);
            }
            Response::Open(path) => {
                open::that(path).unwrap();
            }
            Response::Unhandled => {
                println!("Unhandled Request");
            }
            Response::Ok => {}
            Response::Error(e) => {
                println!("Server Error processing request: {}", e);
                std::process::exit(1);
            }
        }

        Ok(())
    }
}

pub fn print_tabled(metas: Vec<Meta>) {
    use term_table::{
        row::Row,
        table_cell::{Alignment, TableCell},
        Table, TableStyle,
    };

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
            Alignment::Right,
        ));
        row.push(TableCell::new_with_alignment(
            meta.id(),
            1,
            Alignment::Right,
        ));
        table.add_row(Row::new(row));
    }

    print!("{}", table.render());
}

pub async fn check_or_spawn() -> Result<(), std::io::Error> {
    use std::process::{Command, Stdio};
    use sysinfo::{ProcessExt, SystemExt};

    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    if let None = system
        .get_processes()
        .iter()
        .find(|(_pid, proc)| proc.name() == "monkd")
    {
        // println!("Spawning the daemon");

        let _monkd = Command::new("./target/debug/monkd")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()?;

        tokio::time::delay_for(std::time::Duration::from_millis(200)).await;
    }

    Ok(())
}
