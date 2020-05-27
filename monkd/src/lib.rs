#![feature(result_flattening, with_options)]

pub mod daemon;
pub mod error;
pub mod index;
pub mod metadata;
pub mod server;
pub mod settings;

use anyhow::Result;

use crate::daemon::Daemon;
use crate::server::{request::Request, response::Response, Server};
use crate::settings::Settings;

use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

#[tracing::instrument(skip(settings))]
pub async fn run(settings: Settings) -> Result<()> {
    use tokio::stream::StreamExt;

    let start_time = Instant::now();

    let (sender, mut receiver) = mpsc::channel(100);
    let (shutdown, signal) = oneshot::channel::<()>();

    let addr = SocketAddr::new(settings.daemon().address, settings.daemon().port);

    tokio::spawn(Server::spawn(addr, sender, signal));
    let timeout_duration = Duration::from_millis(settings.daemon().timeout as u64);

    let mut daemon = Daemon::new(&settings)?;

    'main: loop {
        let request = timeout(timeout_duration, receiver.next()).await;

        if let Ok(Some((request, response))) = request {
            if let Request::Stop = request {
                tracing::info!("Stop Request Received");
                tracing::info!(
                    "Server is shutting down after {:3.4} s.",
                    start_time.elapsed().as_secs_f32()
                );

                let _ = response.send(Response::Ok);
                let _ = shutdown.send(());
                break 'main;
            }

            let res = daemon.handle_request(request).await?;
            let _ = response.send(res);
        } else if let Ok(None) = request {
            continue;
        } else {
            tracing::info!("Timeout Reached");
            tracing::info!(
                "Server is shutting down after {:3.4} s.",
                start_time.elapsed().as_secs_f32()
            );
            let _ = shutdown.send(());
            break;
        }
    }

    daemon.shutdown().await?;

    Ok(())
}

pub fn generate_id() -> String {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric).to_ascii_lowercase())
        .take(10)
        .collect()
}

// async fn send_commands() {
//     use futures::sink::SinkExt;
//     use std::time::Duration;
//     use tokio::net::TcpStream;
//     use tokio_serde::formats::*;
//     use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
//     use url::Url;

//     tokio::time::delay_for(Duration::from_secs(1)).await;

//     let server = TcpStream::connect("127.0.0.1:8888").await.unwrap();
//     let length_delimited = FramedWrite::new(server, LengthDelimitedCodec::new());

//     let mut serialized = tokio_serde::SymmetricallyFramed::new(
//         length_delimited,
//         SymmetricalJson::<RequestBody>::default(),
//     );

//     for i in 110..113usize {
//         tokio::time::delay_for(Duration::from_secs(1)).await;

//         let body = RequestBody::Add {
//             name: i.to_string(),
//             url: Url::parse("file://request.file/path").unwrap(),
//         };

//         serialized.send(body).await.unwrap();
//     }

//     tokio::time::delay_for(Duration::from_secs(1)).await;

//     serialized.send(RequestBody::Stop).await.unwrap();
// }

// let mut builder: SchemaBuilder = Schema::builder();
//     // let uri = builder.add_text_field("uri", TEXT | STORED);
//     let name = builder.add_text_field("name", TEXT | STORED);
//     let uri = builder.add_text_field("comment", STRING | STORED);
//     let type_ = builder.add_text_field("type", STRING | STORED);
//     let body = builder.add_text_field("body", TEXT | STORED);
//     let comment = builder.add_text_field("comment", TEXT | STORED);
//     let discovered = builder.add_date_field("discovered", INDEXED | STORED);

//     // dated (for specific date to be read on / associated event or time)
//     // stars?
//     // Comments?
//     // body for searching
//     // title?
//     let schema = builder.build();

//     let dir = tantivy::directory::MmapDirectory::open("tantivy")?;
//     let index = Index::open_or_create(dir, schema.clone())?;

//     let now = Utc::now();

//     match args {
//         Args::Add {
//             name: item_name,
//             uri: item_uri,
//             body: item_body,
//             ty,
//             comment: item_comment,
//         } => {
//             let mut index_writer = index.writer(3000000)?;

//             // let ty = ty.unwrap_or_else(|| "article".into());

//             let mut doc = Document::new();

//             doc.add_text(name, &item_name);

//             if let Some(ref item_uri) = item_uri {
//                 doc.add_text(uri, item_uri);
//             }

//             if let Some(ref item_body) = item_body {
//                 doc.add_text(body, item_body);
//             }

//             if let Some(ref ty) = ty {
//                 doc.add_text(type_, ty);
//             }

//             if let Some(ref item_comment) = item_comment {
//                 doc.add_text(comment, item_comment);
//             }

//             let now = Utc::now();
//             doc.add_date(discovered, &now);

//             println!("{:#?}", doc);

//             index_writer.add_document(doc);
//             index_writer.commit()?;
//         }
//         Args::List { count } => {
//             let index_reader = index.reader()?;
//             let searcher = index_reader.searcher();

//             let count = if count == 0 {
//                 searcher.num_docs() as usize
//             } else {
//                 count
//             };

//             if count == 0 {
//                 println!("there are no items");
//                 return Ok(());
//             }

//             let docs: Vec<(f32, DocAddress)> =
//                 searcher.search(&AllQuery, &TopDocs::with_limit(count))?;

//             for (_weight, address) in docs {
//                 let doc = searcher.doc(address)?;

//                 print_document(&doc);
//             }
//         }
//     }

//     Ok(())
