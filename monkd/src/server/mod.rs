pub mod request;
pub mod response;

use self::request::Request;
use self::response::Response;

use std::net::SocketAddr;

use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

use warp::{reply::json, Filter};

pub struct Server;

impl Server {
    pub async fn spawn(
        addr: impl Into<SocketAddr> + 'static,
        sender: Sender<(Request, Option<oneshot::Sender<Response>>)>,
        shutdown: oneshot::Receiver<()>,
    ) {
        let sender = warp::any().map(move || sender.clone());

        let route = warp::any()
            .and(sender)
            .and(warp::body::json())
            .and_then(handle);

        let server = warp::serve(route);

        let (_, server) = server.bind_with_graceful_shutdown(addr, async {
            shutdown.await.ok();
        });

        server.await;
    }
}

#[tracing::instrument]
pub async fn handle(
    mut sender: Sender<(Request, Option<oneshot::Sender<Response>>)>,
    req: Request,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (send, resp) = oneshot::channel();
    sender.send((req, Some(send))).await.map_err(|_| ()).unwrap();

    Ok(match resp.await {
        Ok(r) => json(&r),
        Err(_) => json(&Response::Error(format!("Error receiving oneshot message"))),
    })
}
