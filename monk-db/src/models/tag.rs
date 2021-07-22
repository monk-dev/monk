#![allow(unreachable_code)]

use async_graphql::{ComplexObject, Result as GQLResult, SimpleObject};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use tracing::info;
use uuid::Uuid;

use crate::Error;

use super::article::Article;

static TAG_COLUMNS: &'static str = "id, name, created_at";
static TAG_INSERT_COLUMNS: &'static str = "name, created_at";

pub static TABLE: &'static str = r#"
CREATE TABLE IF NOT EXISTS tag (
    id          UUID    NOT NULL PRIMARY KEY,
    name        STRING  NOT NULL,
    created_at  STRING  NOT NULL
);
"#;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, SimpleObject)]
#[graphql(complex)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl Tag {
    pub fn get(conn: &Connection, id: &Uuid) -> Result<Self, Error> {
        let query = format!("SELECT {} FROM tag WHERE id=?", TAG_COLUMNS);

        conn.prepare(&query)?
            .query_row([id], Tag::from_row)
            .map_err(Into::into)
    }

    pub fn load_ids(conn: &Connection, ids: &[Uuid]) -> Result<Vec<Self>, Error> {
        let mut tags = Vec::new();
        for id in ids {
            tags.push(Tag::get(conn, id)?);
        }

        Ok(tags)
    }

    pub fn insert(name: impl Into<String>) -> InsertTag {
        InsertTag::new(name)
    }

    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Tag {
            id: row.get("id")?,
            name: row.get("name")?,
            created_at: row.get("created_at")?,
        })
    }

    pub fn tags_for_article(conn: &Connection, article_id: &Uuid) -> Result<Vec<Tag>, Error> {
        let query = format!(
            "SELECT {} FROM tag WHERE id in (SELECT tag_id FROM article_tag WHERE article_id=?)",
            TAG_COLUMNS
        );

        Ok(conn
            .prepare(&query)?
            .query_map([article_id], Tag::from_row)?
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub fn create_table(conn: &Connection) -> Result<(), Error> {
        info!("creating table");
        conn.execute(TABLE, [])?;
        Ok(())
    }
}

#[ComplexObject]
impl Tag {
    pub async fn articles(&self) -> GQLResult<Vec<Article>> {
        todo!()
    }
}

pub struct InsertTag {
    name: String,
    created_at: DateTime<Utc>,
}

impl InsertTag {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            created_at: Utc::now(),
        }
    }

    #[tracing::instrument(skip(self, conn), fields(tag.name=%self.name))]
    pub fn execute(self, conn: &Connection) -> Result<Tag, Error> {
        info!("adding tag");

        let query = format!(
            "INSERT INTO tag ({}) VALUES (?, ?) RETURNING {}",
            TAG_INSERT_COLUMNS, TAG_COLUMNS
        );

        conn.prepare(&query)?
            .query_row(params![&self.name, self.created_at], Tag::from_row)
            .map_err(Into::into)
    }
}
