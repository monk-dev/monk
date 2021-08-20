use std::collections::HashSet;

use async_graphql::{ComplexObject, Context, Result as GQLResult, SimpleObject};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use tracing::info;
use url::Url;
use uuid::Uuid;

use crate::{connection::DbConn, mutation::input::UpdateArticleInput, Error};

use super::tag::Tag;

static ARTICLE_COLUMNS: &'static str = "id, user_id, name, description, url, created_at";
static ARTICLE_INSERT_COLUMNS: &'static str = "user_id, name, description, url, created_at";
static ARTICLE_TAG_INSERT_COLUMNS: &'static str = "article_id, tag_id, created_at";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, SimpleObject)]
#[graphql(complex)]
pub struct Article {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub url: Option<Url>,
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

    pub fn add_tag(&self, conn: &Connection, tag_id: &Uuid) -> Result<(), Error> {
        info!("adding tag to article");

        let query = format!(
            "INSERT INTO article_tag ({}) VALUES (?, ?, ?)",
            ARTICLE_TAG_INSERT_COLUMNS,
        );

        conn.prepare(&query)?
            .execute(params![self.id, tag_id, self.created_at])?;

        Ok(())
    }

    pub fn update(conn: &Connection, input: &UpdateArticleInput) -> Result<Self, Error> {
        let query = format!(
            r#"
            UPDATE article
                SET name = COALESCE(?, name),
                    description = COALESCE(?, description),
                    url = COALESCE(?, url)
            WHERE
                id = ?
            RETURNING {};
        "#,
            ARTICLE_COLUMNS
        );

        Ok(conn.prepare(&query)?.query_row(
            params![input.name, input.description, input.url, input.id],
            Article::from_row,
        )?)
    }

    pub fn delete(conn: &Connection, id: &Uuid) -> Result<Self, Error> {
        let query = format!(
            "DELETE FROM article WHERE id=? RETURNING {}",
            ARTICLE_COLUMNS
        );

        Ok(conn.prepare(&query)?.query_row([id], Article::from_row)?)
    }

    pub fn insert(name: impl Into<String>) -> InsertArticle {
        InsertArticle::new(name)
    }

    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Article {
            id: row.get("id")?,
            user_id: row.get("user_id")?,
            name: row.get("name")?,
            url: row.get("url")?,
            description: row.get("description")?,
            created_at: row.get("created_at")?,
        })
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
    user_id: Option<Uuid>,
    name: String,
    description: Option<String>,
    url: Option<Url>,
    tags: HashSet<Uuid>,
    namespaces: HashSet<Uuid>,
    created_at: DateTime<Utc>,
}

impl InsertArticle {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            user_id: None,
            name: name.into(),
            url: None,
            description: None,
            tags: HashSet::new(),
            namespaces: HashSet::new(),
            created_at: Utc::now(),
        }
    }

    pub fn user(mut self, user_id: &Uuid) -> Self {
        self.user_id = Some(user_id.clone());
        self
    }

    pub fn url(mut self, url: Url) -> Self {
        self.url = Some(url);
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn tag(mut self, tag: &Uuid) -> Self {
        self.tags.insert(tag.clone());
        self
    }

    pub fn tags(mut self, tags: &[Uuid]) -> Self {
        self.tags.extend(tags.iter());
        self
    }

    pub fn namespace(mut self, namespace: &Uuid) -> Self {
        self.namespaces.insert(namespace.clone());
        self
    }

    pub fn namespaces(mut self, namespaces: &[Uuid]) -> Self {
        self.namespaces.extend(namespaces.iter());
        self
    }

    #[tracing::instrument(skip(self, conn), fields(article.name=%self.name))]
    pub fn execute(self, conn: &Connection) -> Result<Article, Error> {
        info!("adding article");

        let query = format!(
            "INSERT INTO article ({}) VALUES (?, ?, ?, ?, ?) RETURNING {}",
            ARTICLE_INSERT_COLUMNS, ARTICLE_COLUMNS
        );

        let article = conn.prepare(&query)?.query_row(
            params![
                self.user_id,
                self.name,
                self.description,
                self.url,
                self.created_at
            ],
            Article::from_row,
        )?;

        info!(article=%article.id, "inserted");
        for tag_id in self.tags {
            article.add_tag(conn, &tag_id)?;
        }

        Ok(article)
    }
}
