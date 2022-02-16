pub mod errors;

use axum::error_handling::HandleError;
use axum::extract::Extension;
use axum::AddExtensionLayer;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use monk::types::config::MonkConfig;
use monk::types::{AddItem, Item, MonkTrait};
use monk::Monk;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{any, Any, CorsLayer, Origin};

use self::errors::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct State {
    monk: Arc<Mutex<Monk>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let config = MonkConfig::default();
    let monk = Monk::from_config(config).await?;

    let state = State {
        monk: Arc::new(Mutex::new(monk)),
    };

    // build our application with a route
    let app = Router::new()
        .route("/items", get(list_items))
        .route("/items", post(create_item))
        .layer(AddExtensionLayer::new(state))
        .layer(CorsLayer::new().allow_origin(any()).allow_methods(any()));
    // `GET /` goes to `root`
    // .route("/", get(root))
    // // `POST /users` goes to `create_user`
    // .route("/users", post(create_user));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn list_items(Extension(state): Extension<State>) -> Result<Json<Vec<Item>>> {
    let mut monk = state.monk.lock().await;
    let items = monk.list(Default::default()).await?;

    Ok(Json(items))
}

async fn create_item(
    Extension(state): Extension<State>,
    Json(item): Json<AddItem>,
) -> Result<Json<Item>> {
    let mut monk = state.monk.lock().await;
    let item = monk.add(item).await?;

    Ok(Json(item))
}
