use std::net::SocketAddr;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};
use url::Url;

use crate::args::{Args, Subcommand};
use crate::error::Error;

use monkd::metadata::Meta;
use monkd::server::{request::Request, response::Response};
use monkd::settings::Settings;

pub struct Cli;

impl Cli {
    pub async fn run(settings: Settings, args: Args) -> Result<(), Error> {
        check_or_spawn(&settings).await?;

        let request = match args.subcommand {
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
            Subcommand::Add { name, url, comment } => {
                if name.is_none() && url.is_none() && comment.is_none() {
                    println!("either name, url, or comment must be set");
                    std::process::exit(1);
                }

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
            Response::TooManyMeta(id, metas) => print_too_many(id, metas),
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
                println!("error: {}", e);
                std::process::exit(1);
            }
        }

        Ok(())
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

        // if let Some(last_read) = meta.last_read() {
        //     row.push(TableCell::new_with_alignment(
        //         last_read.format("%b %d, %Y").to_string(),
        //         1,
        //         Alignment::Center,
        //     ));
        // } else {
        //     row.push(TableCell::new_with_alignment("n/a", 1, Alignment::Center));
        // }

        table.add_row(Row::new(row));
    }

    table
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