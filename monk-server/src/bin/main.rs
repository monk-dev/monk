use monk_db::init_db;

// use std::env;

// use matrix_sdk::uuid::Uuid;
// use monk_server::matrix::login_and_sync;
// use rusqlite::{params, Connection};
// use tracing::*;

// use monk_db::Article;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let conn = init_db("monk.db")?;
    println!("{:?}", conn);

    Ok(())
}
