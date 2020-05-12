use anyhow::Result;
use futures::prelude::*;
use std::net::IpAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    oneshot,
};
use tokio_serde::formats::*;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tracing::{error, event, span, Level};

use crate::error::Error;
use crate::request::{CliRequest, RequestBody, ResponseBody};

pub struct Server;

impl Server {
    pub async fn spawn(
        addr: impl ToSocketAddrs + std::fmt::Debug,
    ) -> Result<Receiver<Result<CliRequest, Error>>> {
        let (s, r) = channel(1024);

        let listener = TcpListener::bind(&addr).await?;
        tokio::spawn(Server::run(listener, s));

        event!(Level::INFO, "Server connected to: {:?}", addr);

        Ok(r)
    }

    async fn run(
        mut listener: TcpListener,
        mut sender: Sender<Result<CliRequest, Error>>,
    ) -> Result<()> {
        let span = span!(Level::TRACE, "Server Loop");
        let _enter = span.enter();

        while let Some(stream) = listener.next().await {
            match stream {
                Ok(stream) => {
                    event!(Level::INFO, "New client: {:?}", stream.peer_addr());
                    handle_stream(stream, sender.clone())
                }
                Err(e) => {
                    event!(Level::ERROR, "Listener error: {:?}", e);
                    sender.send(Err(e.into())).await?;
                }
            }
        }

        Ok(())
    }
}

// #[instrument(level = "debug")]
fn handle_stream(stream: TcpStream, mut sender: Sender<Result<CliRequest, Error>>) {
    let (read_half, write_half) = stream.into_split();

    let length_delimited_read = FramedRead::new(read_half, LengthDelimitedCodec::new());
    let mut deserialized = tokio_serde::SymmetricallyFramed::new(
        length_delimited_read,
        SymmetricalJson::<RequestBody>::default(),
    );

    let length_delimited_write = FramedWrite::new(write_half, LengthDelimitedCodec::new());
    let mut serialized = tokio_serde::SymmetricallyFramed::new(
        length_delimited_write,
        SymmetricalJson::<ResponseBody>::default(),
    );

    tokio::spawn(async move {
        while let Some(msg) = deserialized.next().await {
            let _ = match msg {
                Ok(msg) => {
                    if msg.gives_response() {
                        let (s, rx) = oneshot::channel();

                        let cli_req = CliRequest::new(msg, Some(s));

                        let mut s = sender.clone();
                        tokio::spawn(async move { s.send(Ok(cli_req)).await });

                        let resp = rx
                            .await
                            .map_err(Error::from)
                            .map(|e| e.map_err(Error::from))
                            .flatten()
                            .map_err(|e| ResponseBody::Error(e.to_string()))
                            .unwrap();

                        serialized.send(resp).await.map_err(Error::from)
                    } else {
                        sender
                            .send(Ok(CliRequest::new(msg, None)))
                            .await
                            .map_err(|e| Error::Custom(e.to_string()))
                    }
                }
                Err(e) => sender
                    .send(Err(e.into()))
                    .await
                    .map_err(|e| Error::Custom(e.to_string())),
            }
            .map_err(|e| error!("Message send error: {}", e));
        }
    });
}
