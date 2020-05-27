use std::net::SocketAddr;

use crate::args::{Args, Subcommand};
use crate::error::Error;

use monkd::server::{request::Request, response::Response};
use monkd::settings::Settings;
use url::Url;

pub struct Cli;

impl Cli {
    pub async fn run(settings: Settings, args: Args) -> Result<(), Error> {
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
                for (i, meta) in items.iter().enumerate() {
                    println!("[{}]: {}", i, meta);
                }
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

pub fn check_or_spawn() -> Result<(), std::io::Error> {
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

        let _monkd = Command::new("monkd")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()?;
    }

    Ok(())
}
