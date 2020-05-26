use std::net::SocketAddr;

use crate::args::{Args, Subcommand};
use crate::error::Error;
use crate::settings::Settings;

use daemon::server::{request::Request, response::Response};
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

        let socket = SocketAddr::new(*settings.address(), settings.port());
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
                println!("{:?}", meta);
            }
            Response::List(items) => {
                for meta in items {
                    println!("{:?}", meta);
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
            Response::Ok => {
                println!("Request was successful");
            }
            Response::Error(e) => {
                println!("Server Error processing request:\n{}", e);
            }
        }

        Ok(())
    }
}
