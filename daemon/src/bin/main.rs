use daemon::metadata::{Meta, MetaStore};
use daemon::request::Request;
use daemon::server::Server;

use std::fs::File;
use std::io::BufWriter;

use anyhow::Result;
use chrono::Utc;
use serde_json::Serializer;
use tokio::{main, stream::StreamExt};

async fn send_commands() {
    use futures::sink::SinkExt;
    use std::time::Duration;
    use tokio::net::TcpStream;
    use tokio_serde::formats::*;
    use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
    use url::Url;

    tokio::time::delay_for(Duration::from_secs(1)).await;

    let server = TcpStream::connect("127.0.0.1:8888").await.unwrap();
    let length_delimited = FramedWrite::new(server, LengthDelimitedCodec::new());

    let mut serialized = tokio_serde::SymmetricallyFramed::new(
        length_delimited,
        SymmetricalJson::<Request>::default(),
    );

    for i in 110..113usize {
        tokio::time::delay_for(Duration::from_secs(1)).await;

        let request = Request::Add {
            name: i.to_string(),
            url: Url::parse("file://request.file/path").unwrap(),
        };

        serialized.send(request).await.unwrap();
    }

    tokio::time::delay_for(Duration::from_secs(1)).await;

    serialized.send(Request::Stop).await.unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut store = MetaStore::empty();

    for i in 0..2 {
        let meta = Meta::new(i.to_string(), "file://file.com/file", Utc::now())?;
        store.push(meta);
    }

    let _ = tokio::task::spawn_blocking(move || store.write_file("store.json")).await?;

    let mut store = tokio::task::spawn_blocking(|| MetaStore::read_file("store.json")).await??;

    let mut rx = Server::spawn("127.0.0.1:8888").await?;

    tokio::spawn(send_commands());

    while let Some(msg) = rx.next().await {
        println!("Received: {:?}", msg);

        match msg? {
            Request::Add { name, url } => {
                let now = Utc::now();
                store.push(Meta::new(name, url.as_str(), now)?);
            }
            l @ Request::List { .. } => {
                println!("Received a {:?}", l);
            }
            Request::Stop => {
                break;
            }
        }
    }

    println!("{:#?}", store);

    Ok(())
}
