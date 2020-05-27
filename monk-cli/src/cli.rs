use std::net::SocketAddr;

use crate::args::{Args, Subcommand};
use crate::error::Error;

use monkd::server::{request::Request, response::Response};
use monkd::settings::Settings;
use monkd::metadata::Meta;
use url::Url;

pub struct Cli;

impl Cli {
    pub async fn run(settings: Settings, args: Args) -> Result<(), Error> {
        check_or_spawn().await?;
        
        let request = match args.subcommand {
            Subcommand::Add { name, url } => {
                let url: Option<Url> = url.map(|s| Url::parse(&s)).transpose()?;

                Request::Add {
                    name: Some(name.clone()),
                    url,
                }
            }
            Subcommand::List { count } => Request::List { count },
            Subcommand::Delete { id } => Request::Delete { id },
            Subcommand::Get { id } => Request::Get { id },
            Subcommand::Stop => Request::Stop,
            Subcommand::ForceShutdown => Request::ForceShutdown,
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
                println!("{}", meta);
            }
            Response::List(items) => {
                print_tabled(items);
                // for (i, meta) in items.iter().enumerate() {
                //     println!("[{}]: {}", i, meta);
                // }
            }
            Response::NewId(id) => {
                println!("Created ID: {}", id);
            }
            Response::NotFound(id) => {
                println!("ID not found: {}", id);
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
    use term_table::{Table, TableStyle, row::Row, table_cell::{TableCell, Alignment}};

    let mut table = Table::new();
    table.max_column_width = 40;
    table.style = TableStyle::extended();

    for meta in metas {
        let mut row = Vec::new();

        row.push(TableCell::new_with_alignment(meta.name().unwrap_or(""), 1, Alignment::Left));
        row.push(TableCell::new_with_alignment(meta.url().map(|u| u.to_string()).unwrap_or("".into()), 1, Alignment::Left));
        row.push(TableCell::new_with_alignment(meta.comment().unwrap_or("n/a".into()), 1, Alignment::Left));
        row.push(TableCell::new_with_alignment(meta.found().format("%b %d, %Y").to_string(), 1, Alignment::Right));
        row.push(TableCell::new_with_alignment(meta.id(), 1, Alignment::Right));

        table.add_row(Row::new(row));

        // write!(f, "[{}]", meta.id())?;

        // if let Some(name) = meta.name() {
        //     write!(f, " {}:", name)?;
        // } else {
        //     write!(f, "n/a:")?;
        // }

        // if let Some(url) = meta.url() {
        //     write!(f, " {}", url.to_string())?;
        // }

        // let found = meta.found.format("%a %d, %Y").to_string();
        // write!(f, " @ {}", found)?;

        // if let Some(comment) = meta.comment() {
        //     write!(f, "\n\t{}", comment)?;
        // }
    }

    println!("{}", table.render());
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
        println!("Spawning the daemon");

        let _monkd = Command::new("./target/debug/monkd")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()?;

        tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
    }

    Ok(())
}
