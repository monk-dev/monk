use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use tracing::info;
use uuid::Uuid;

use crate::Error;

static ARTICLE_TAG_COLUMNS: &'static str = "id, article_id, tag_id, created_at";

pub static TABLE: &'static str = r#"
CREATE TABLE IF NOT EXISTS article_tag (
    id                      UUID    NOT NULL PRIMARY KEY,
    article_id              UUID REFERENCES article(id) ON UPDATE CASCADE,
    tag_id                  UUID REFERENCES tag(id) ON UPDATE CASCADE,
    created_at              STRING  NOT NULL
);
"#;

pub struct ArticleTag;

impl ArticleTag {
    pub fn create_table(conn: &Connection) -> Result<(), Error> {
        info!("Creating Table: ARTICLE_TAG");
        conn.execute(TABLE, [])?;
        Ok(())
    }
}

pub struct AddTagToArticle<'a, 't> {
    id: Uuid,
    article_id: &'a Uuid,
    tag_id: &'t Uuid,
    created_at: DateTime<Utc>,
}

impl<'a, 't> AddTagToArticle<'a, 't> {
    pub fn new(article_id: &'a Uuid, tag_id: &'t Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            article_id,
            tag_id,
            created_at: Utc::now(),
        }
    }

    pub fn create_table(conn: &Connection) -> Result<(), Error> {
        info!("creating table");
        conn.execute(TABLE, [])?;
        Ok(())
    }
}

impl<'a, 't> AddTagToArticle<'a, 't> {
    #[tracing::instrument(skip(self, conn), fields(article.id=%self.article_id, tag.id=%self.tag_id))]
    pub fn execute(self, conn: &Connection) -> Result<(), Error> {
        info!("adding tag to article");

        let query = format!(
            "INSERT INTO article_tag ({}) VALUES (?, ?, ?, ?)",
            ARTICLE_TAG_COLUMNS,
        );

        conn.prepare(&query)?.execute(params![
            &self.id,
            self.article_id,
            self.tag_id,
            self.created_at
        ])?;

        Ok(())
    }
}
