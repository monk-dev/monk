// pub mod context;
pub mod connection;
pub mod error;
pub mod graphql;
pub mod models;
pub mod mutation;
pub mod query;

use std::path::Path;

use connection::DbConn;
use rusqlite::{functions::FunctionFlags, Connection};
use tracing::info;
use uuid::Uuid;

pub use crate::error::Error;
use crate::models::{article::Article, tag::Tag};

static SCHEMA: &'static str = include_str!("./schema.sql");

#[tracing::instrument(skip(path), fields(db_path=%path.as_ref().display()))]
pub fn init_db(path: impl AsRef<Path>) -> Result<DbConn, Error> {
    let conn = Connection::open(path)?;

    add_uuid_function(&conn)?;

    info!("creating schema");
    conn.execute_batch(&SCHEMA)?;

    #[cfg(debug_assertions)]
    seed(&conn)?;

    let count = conn.execute("INSERT INTO article (name) VALUES (\"It's a name!\")", [])?;
    info!("count: {}", count);

    Ok(DbConn::new(conn))
}

#[cfg(debug_assertions)]
#[tracing::instrument]
pub fn seed(conn: &Connection) -> Result<(), Error> {
    use crate::models::{namespace::Namespace, user::User};

    info!("seeding db");

    info!("deleting old rows");
    conn.execute("DELETE FROM article_tag", [])?;
    conn.execute("DELETE FROM tag", [])?;
    conn.execute("DELETE FROM article", [])?;
    conn.execute("DELETE FROM namespace_article", [])?;
    conn.execute("DELETE FROM namespace", [])?;
    conn.execute("DELETE FROM user", [])?;

    let default_user = User::insert("default").execute(&conn)?;

    let default_namespace = Namespace::insert("default", &default_user.id).execute(&conn)?;

    let linux = Tag::insert("linux").execute(&conn)?;
    let rust = Tag::insert("rust").execute(&conn)?;

    let af_xdp = Article::insert("AF_XDP")
        .url("https://lwn.net/Articles/750845/".parse().unwrap())
        .description("Super fast packet capturing")
        .tag(&linux.id)
        .execute(&conn)?;

    let aya = Article::insert("Aya: eBPFs In Rust")
        .url("https://github.com/alessandrod/aya".parse().unwrap())
        .tags(&[linux.id, rust.id])
        .execute(&conn)?;

    default_namespace.add_articles(conn, &[af_xdp.id, aya.id])?;

    Ok(())
}

// fn add_regexp_function(db: &Connection) -> Result<()> {
//     db.create_scalar_function(
//         "regexp",
//         2,
//         FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
//         move |ctx| {
//             assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");
//             let regexp: Arc<Regex> = ctx.get_or_create_aux(0, |vr| -> Result<_, BoxError> {
//                 Ok(Regex::new(vr.as_str()?)?)
//             })?;
//             let is_match = {
//                 let text = ctx
//                     .get_raw(1)
//                     .as_str()
//                     .map_err(|e| Error::UserFunctionError(e.into()))?;

//                 regexp.is_match(text)
//             };

//             Ok(is_match)
//         },
//     )
// }

fn add_uuid_function(db: &Connection) -> Result<(), Error> {
    db.create_scalar_function("uuid", 0, FunctionFlags::empty(), |_| {
        let id = Uuid::new_v4();
        Ok(id.as_bytes().to_vec())
    })?;

    Ok(())
}
