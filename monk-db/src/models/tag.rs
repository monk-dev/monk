use async_graphql::{ComplexObject, Result as GQLResult, SimpleObject};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use tracing::info;
use uuid::Uuid;

use crate::Error;

use super::article::Article;

static TAG_COLUMNS: &'static str = "id, name";

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
            "SELECT {} FROM tag t INNER JOIN article a on t.id=a.article_id WHERE a.id=?",
            TAG_COLUMNS
        );

        Ok(conn
            .prepare(&query)?
            .query_map([article_id], Tag::from_row)?
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub fn create_table(conn: &Connection) -> Result<(), Error> {
        info!("Creating Table: tag");
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
    id: Uuid,
    name: String,
    created_at: DateTime<Utc>,
}

impl InsertTag {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            created_at: Utc::now(),
        }
    }

    #[tracing::instrument(skip(self, conn), fields(tag.id=%self.id, tag.name=%self.name))]
    pub fn execute(self, conn: &Connection) -> Result<Tag, Error> {
        info!("adding tag");

        let query = format!(
            "INSERT INTO tag ({}) VALUES (?, ?, ?, ?) RETURNING {}",
            TAG_COLUMNS, TAG_COLUMNS,
        );

        conn.prepare(&query)?
            .query_row(params![&self.id, self.name, self.created_at], Tag::from_row)
            .map_err(Into::into)
    }
}
