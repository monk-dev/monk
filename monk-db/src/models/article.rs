use async_graphql::{ComplexObject, Context, Result as GQLResult, SimpleObject};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use tracing::info;
use url::Url;
use uuid::Uuid;

use crate::{connection::DbConn, models::article_tag::AddTagToArticle, Error};

use super::tag::Tag;

static ARTICLE_COLUMNS: &'static str = "id, name, description, url, created_at";

pub static TABLE: &'static str = r#"
CREATE TABLE IF NOT EXISTS article (
    id          UUID    NOT NULL PRIMARY KEY,
    name        STRING  NOT NULL,
    description STRING,
    url         URL,
    created_at  INT     NOT NULL
);
"#;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, SimpleObject)]
#[graphql(complex)]
pub struct Article {
    pub id: Uuid,
    pub name: String,
    pub url: Url,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Article {
    pub fn get(conn: &Connection, id: &Uuid) -> Result<Self, Error> {
        let query = format!("SELECT {} FROM article WHERE id=?", ARTICLE_COLUMNS);

        conn.prepare(&query)?
            .query_row([id], Article::from_row)
            .map_err(Into::into)
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>, Error> {
        let query = format!("SELECT {} FROM article", ARTICLE_COLUMNS);

        Ok(conn
            .prepare(&query)?
            .query_map([], Article::from_row)?
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Article {
            id: row.get("id")?,
            name: row.get("name")?,
            url: row.get("url")?,
            description: row.get("description")?,
            created_at: row.get("created_at")?,
        })
    }

    pub fn insert(name: impl Into<String>) -> InsertArticle {
        InsertArticle::new(name)
    }

    pub fn create_table(conn: &Connection) -> Result<(), Error> {
        info!("Creating Table: article");
        conn.execute(TABLE, [])?;
        Ok(())
    }
}

#[ComplexObject]
impl Article {
    pub async fn tags(&self, ctx: &Context<'_>) -> GQLResult<Vec<Tag>> {
        let conn = ctx.data::<DbConn>()?.get().await;
        Tag::tags_for_article(&conn, &self.id).map_err(Into::into)
    }
}

pub struct InsertArticle {
    id: Uuid,
    name: String,
    description: Option<String>,
    url: Option<Url>,
    tags: Vec<Tag>,
    created_at: DateTime<Utc>,
}

impl InsertArticle {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            url: None,
            description: None,
            tags: Vec::new(),
            created_at: Utc::now(),
        }
    }

    pub fn url(mut self, url: Url) -> Self {
        self.url = Some(url);
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn tag(mut self, tag: &Tag) -> Self {
        self.tags.push(tag.clone());
        self
    }

    pub fn tags(mut self, tags: &[Tag]) -> Self {
        self.tags.extend_from_slice(tags);
        self
    }

    #[tracing::instrument(skip(self, conn), fields(article.id=%self.id, article.name=%self.name))]
    pub fn execute(self, conn: &Connection) -> Result<Article, Error> {
        info!("adding article");

        let query = format!(
            "INSERT INTO article ({}) VALUES (?, ?, ?, ?, ?) RETURNING {}",
            ARTICLE_COLUMNS, ARTICLE_COLUMNS,
        );

        let inserted = conn.prepare(&query)?.query_row(
            params![
                &self.id,
                self.name,
                self.description,
                self.url,
                self.created_at
            ],
            Article::from_row,
        )?;

        for tag in self.tags {
            AddTagToArticle::new(&self.id, &tag.id).execute(conn)?;
        }

        Ok(inserted)
    }
}
