pub mod errors;

use axum::extract::Extension;
use axum::AddExtensionLayer;
use axum::{
    routing::{get, post},
    Json, Router,
};
use monk::types::config::MonkConfig;
use monk::types::{AddItem, Item, MonkTrait};
use monk::Monk;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{any, CorsLayer};

use self::errors::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct State {
    monk: Arc<Mutex<Monk>>,
}

pub async fn run(config: MonkConfig) -> anyhow::Result<Arc<Mutex<Monk>>> {
    let monk = Arc::new(Mutex::new(Monk::from_config(config).await?));

    let state = State {
        monk: Arc::clone(&monk),
    };

    // build our application with a route
    let app = Router::new()
        .route("/items", get(list_items))
        .route("/items", post(create_item))
        .layer(AddExtensionLayer::new(state))
        .layer(
            CorsLayer::new()
                .allow_origin(any())
                .allow_methods(any())
                .allow_headers(any()),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::info!("listening on {}", addr);
    tokio::spawn(async move {
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(monk)
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
