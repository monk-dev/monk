use anyhow::Result;
use std::net::IpAddr;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    stream::StreamExt,
};
use tokio_serde::formats::*;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

use crate::error::Error;
use crate::request::Request;

pub struct Server;

fn handle_stream(stream: TcpStream, mut sender: Sender<Result<Request, Error>>) {
    let length_delimited = FramedRead::new(stream, LengthDelimitedCodec::new());

    let mut deserialized = tokio_serde::SymmetricallyFramed::new(
        length_delimited,
        SymmetricalJson::<Request>::default(),
    );

    tokio::spawn(async move {
        while let Some(msg) = deserialized.next().await {
            let _ = match msg {
                Ok(msg) => sender.send(Ok(msg)).await,
                Err(e) => sender.send(Err(e.into())).await,
            }
            .map_err(|e| eprintln!("Message send error: {}", e));
        }
    });
}

impl Server {
    pub async fn spawn(addr: impl ToSocketAddrs) -> Result<Receiver<Result<Request, Error>>> {
        let (s, r) = channel(1024);

        let listener = TcpListener::bind(addr).await?;
        tokio::spawn(Server::run(listener, s));

        Ok(r)
    }

    async fn run(
        mut listener: TcpListener,
        mut sender: Sender<Result<Request, Error>>,
    ) -> Result<()> {
        while let Some(stream) = listener.next().await {
            match stream {
                Ok(stream) => handle_stream(stream, sender.clone()),
                Err(e) => {
                    sender.send(Err(e.into())).await?;
                }
            }
        }

        Ok(())
    }
}
