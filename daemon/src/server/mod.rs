pub mod request;
pub mod response;

use self::request::Request;
use self::response::Response;
use crate::error::Error;

use std::net::SocketAddr;

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;

use warp::{
    http::Response as HttpResponse,
    reply::{json, Json},
    Filter,
};

pub struct Server;

impl Server {
    pub async fn spawn(
        addr: impl Into<SocketAddr> + 'static,
        sender: Sender<(Request, oneshot::Sender<Response>)>,
        shutdown: oneshot::Receiver<()>,
    ) {
        let sender = warp::any().map(move || sender.clone());

        let add = warp::path("add")
            .and(sender.clone())
            .and(warp::body::json())
            .and_then(add);

        let list = warp::path("list")
            .and(sender.clone())
            .and(warp::body::json())
            .and_then(list);

        let stop = warp::path("stop")
            .and(sender.clone())
            .and(warp::body::json())
            .and_then(stop);

        let routes = add.or(list).or(stop);

        let server = warp::serve(routes);

        let (_, server) = server.bind_with_graceful_shutdown(addr, async {
            shutdown.await.ok();
        });

        server.await;
    }
}

#[tracing::instrument]
pub async fn add(
    mut sender: Sender<(Request, oneshot::Sender<Response>)>,
    req: Request,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (send, resp) = oneshot::channel();
    sender.send((req, send)).await.map_err(|_| ()).unwrap();

    Ok(match resp.await {
        Ok(r) => json(&r),
        Err(_) => json(&Response::Error),
    })
}

#[tracing::instrument]
pub async fn list(
    mut sender: Sender<(Request, oneshot::Sender<Response>)>,
    req: Request,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (send, resp) = oneshot::channel();
    sender.send((req, send)).await.map_err(|_| ()).unwrap();

    Ok(match resp.await {
        Ok(r) => json(&r),
        Err(_) => json(&Response::Error),
    })
}

#[tracing::instrument]
pub async fn stop(
    mut sender: Sender<(Request, oneshot::Sender<Response>)>,
    req: Request,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (send, resp) = oneshot::channel();
    sender.send((req, send)).await.map_err(|_| ()).unwrap();

    Ok(match resp.await {
        Ok(r) => json(&r),
        Err(_) => json(&Response::Error),
    })
}
