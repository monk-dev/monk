#![allow(unreachable_code)]

use async_graphql::{ComplexObject, Result as GQLResult, SimpleObject};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use tracing::info;
use uuid::Uuid;

use crate::Error;

static NAMESPACE_COLUMNS: &'static str = "id, user_id, name, created_at";
static NAMESPACE_INSERT_COLUMNS: &'static str = "user_id, name";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, SimpleObject)]
// #[graphql(complex)]
pub struct Namespace {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl Namespace {
    pub fn get(conn: &Connection, id: &Uuid) -> Result<Self, Error> {
        let query = format!("SELECT {} FROM user WHERE id=?", NAMESPACE_COLUMNS);

        conn.prepare(&query)?
            .query_row([id], Namespace::from_row)
            .map_err(Into::into)
    }

    pub fn insert(name: impl Into<String>, user_id: &Uuid) -> InsertNamespace {
        InsertNamespace::new(name, user_id.clone())
    }

    pub fn add_article(&self, conn: &Connection, article_id: &Uuid) -> Result<(), Error> {
        let query = "INSERT INTO namespace_article (namespace_id, article_id) VALUES (?, ?)";
        conn.prepare(query)?.execute([&self.id, article_id])?;

        Ok(())
    }

    pub fn add_articles(&self, conn: &Connection, article_ids: &[Uuid]) -> Result<(), Error> {
        for article_id in article_ids {
            self.add_article(conn, article_id)?;
        }

        Ok(())
    }

    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Namespace {
            id: row.get("id")?,
            user_id: row.get("user_id")?,
            name: row.get("name")?,
            created_at: row.get("created_at")?,
        })
    }
}

pub struct InsertNamespace {
    name: String,
    user_id: Uuid,
}

impl InsertNamespace {
    pub fn new(name: impl Into<String>, user_id: Uuid) -> InsertNamespace {
        InsertNamespace {
            name: name.into(),
            user_id,
        }
    }

    #[tracing::instrument(skip(self, conn), fields(name=%self.name, user.id=%self.user_id))]
    pub fn execute(self, conn: &Connection) -> Result<Namespace, Error> {
        info!("adding namespace");

        let query = format!(
            "INSERT INTO namespace ({}) VALUES (?, ?) RETURNING {}",
            NAMESPACE_INSERT_COLUMNS, NAMESPACE_COLUMNS,
        );

        conn.prepare(&query)?
            .query_row(params![self.user_id, self.name], Namespace::from_row)
            .map_err(Into::into)
    }
}
