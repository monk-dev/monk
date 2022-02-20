mod entities;
mod store;

use std::str::FromStr;

use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};
use sqlx::ConnectOptions;
use tracing::info;

use crate::entities::{blob, item, item_tag, link, tag};

pub use self::store::MonkSqlite;

async fn create_models(path: &str) -> anyhow::Result<()> {
    let db = Database::connect(path).await?;
    let db_sqlite = DbBackend::Sqlite;
    let schema = Schema::new(db_sqlite);

    // ==== Item ====
    let sql = db_sqlite.build(
        schema
            .create_table_from_entity(item::Entity)
            .if_not_exists(),
    );
    info!(%sql, "item sql");
    db.execute(sql).await?;

    // ==== Tag ====
    let sql = db_sqlite.build(schema.create_table_from_entity(tag::Entity).if_not_exists());
    info!(%sql, "tag sql");
    db.execute(sql).await?;

    // ==== Item <-> Tag ====
    let sql = db_sqlite.build(
        schema
            .create_table_from_entity(item_tag::Entity)
            .if_not_exists(),
    );
    info!(%sql, "item_tag sql");
    db.execute(sql).await?;

    // ==== Blob ====
    let sql = db_sqlite.build(
        schema
            .create_table_from_entity(blob::Entity)
            .if_not_exists(),
    );
    info!(%sql, "blob sql");
    db.execute(sql).await?;

    // ==== Link: Item <-> Item ====
    let sql = db_sqlite.build(
        schema
            .create_table_from_entity(link::Entity)
            .if_not_exists(),
    );
    info!(%sql, "link sql");
    db.execute(sql).await?;

    Ok(())
}

async fn run_migrations(path: &str) -> anyhow::Result<()> {
    let mut conn = sqlx::sqlite::SqliteConnectOptions::from_str(path)?
        .connect()
        .await?;

    sqlx::migrate!().run(&mut conn).await?;

    Ok(())
}
