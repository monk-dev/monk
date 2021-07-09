// pub mod context;
pub mod connection;
pub mod error;
pub mod graphql;
pub mod models;
pub mod query;

use std::path::Path;

use connection::DbConn;
use rusqlite::Connection;
use tracing::info;

pub use crate::error::Error;
use crate::models::{article::Article, article_tag::ArticleTag, tag::Tag};

#[tracing::instrument(skip(path), fields(db_path=%path.as_ref().display()))]
pub fn init_db(path: impl AsRef<Path>) -> Result<DbConn, Error> {
    let conn = Connection::open(path)?;

    Article::create_table(&conn)?;
    Tag::create_table(&conn)?;
    ArticleTag::create_table(&conn)?;

    #[cfg(debug_assertions)]
    seed(&conn)?;

    Ok(DbConn::new(conn))
}

#[cfg(debug_assertions)]
#[tracing::instrument]
pub fn seed(conn: &Connection) -> Result<(), Error> {
    info!("seeding db");

    info!("deleting old rows");
    conn.execute("DELETE FROM article_tag", [])?;
    conn.execute("DELETE FROM tag", [])?;
    conn.execute("DELETE FROM article", [])?;

    let linux = Tag::insert("linux").execute(&conn)?;
    let rust = Tag::insert("rust").execute(&conn)?;

    let _af_xdp = Article::insert("AF_XDP")
        .url("https://lwn.net/Articles/750845/".parse().unwrap())
        .description("Super fast packet capturing")
        .tag(&linux)
        .execute(&conn)?;

    let _aya = Article::insert("Aya: eBPFs In Rust")
        .url("https://github.com/alessandrod/aya".parse().unwrap())
        .tags(&[linux, rust])
        .execute(&conn)?;

    Ok(())
}
