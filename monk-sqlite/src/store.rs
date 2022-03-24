use std::path::Path;

use anyhow::Context;
use monk_types::config::StoreConfig;
use monk_types::{Blob, Item, Store, Tag};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter, Set,
};
use sqlx::types::chrono::Utc;
use uuid::Uuid;

use crate::entities::{blob, item, item_tag, link, tag};

pub struct MonkSqlite {
    db: DatabaseConnection,
}

impl MonkSqlite {
    pub async fn from_config(
        data_dir: impl AsRef<Path>,
        config: &StoreConfig,
    ) -> anyhow::Result<Self> {
        let path = data_dir.as_ref().join(&config.path).display().to_string();

        // Causes sqlx to create the database if it does not exist
        let path = format!("{path}?mode=rwc");

        crate::create_models(&path).await?;
        crate::run_migrations(&path).await?;

        let db = Database::connect(path).await?;

        Ok(Self { db })
    }

    pub async fn get_item_model(&self, id: Uuid) -> anyhow::Result<Option<item::Model>> {
        item::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .context("getting single item")
    }

    pub async fn enrich_item(&self, item: item::Model) -> anyhow::Result<Item> {
        let tags = item
            .find_related(tag::Entity)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        let blob = item
            .find_related(blob::Entity)
            .one(&self.db)
            .await?
            .map(Into::into);

        Ok(Item {
            id: item.id,
            name: item.name,
            url: item.url,
            body: item.body,
            comment: item.comment,
            summary: item.summary,
            tags,
            blob,
            created_at: item.created_at,
        })
    }

    pub async fn get_blob_model(&self, id: Uuid) -> anyhow::Result<Option<blob::Model>> {
        blob::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .context("getting single blob")
    }

    pub async fn get_or_insert_tag(&self, tag: String) -> anyhow::Result<tag::Model> {
        let tag_opt = tag::Entity::find()
            .filter(tag::Column::Tag.eq(tag.as_str()))
            .one(&self.db)
            .await?;

        let tag = if let Some(t) = tag_opt {
            t
        } else {
            tag::ActiveModel {
                id: Set(Uuid::new_v4()),
                tag: Set(tag),
                created_at: Set(Utc::now()),
            }
            .insert(&self.db)
            .await?
        };

        Ok(tag)
    }
}

#[async_trait::async_trait]
impl Store for MonkSqlite {
    async fn item(&self, id: Uuid) -> anyhow::Result<Option<Item>> {
        if let Some(item) = self.get_item_model(id).await? {
            self.enrich_item(item).await.map(Some)
        } else {
            Ok(None)
        }
    }

    async fn list_items(&self) -> anyhow::Result<Vec<Item>> {
        let item_models = item::Entity::find().all(&self.db).await?;

        let mut items = Vec::with_capacity(item_models.len());

        for item_model in item_models {
            items.push(self.enrich_item(item_model).await?);
        }

        Ok(items)
    }

    async fn add_item(
        &self,
        name: String,
        url: Option<String>,
        comment: Option<String>,
        tags: Vec<String>,
    ) -> anyhow::Result<Item> {
        let item = item::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name),
            url: Set(url),
            comment: Set(comment),
            created_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&self.db)
        .await?;

        for tag in tags {
            let tag = self.get_or_insert_tag(tag).await?;

            item_tag::ActiveModel {
                item_id: Set(item.id),
                tag_id: Set(tag.id),
            }
            .insert(&self.db)
            .await?;
        }

        let item = self.enrich_item(item).await?;
        Ok(item)
    }

    async fn delete_item(&self, id: Uuid) -> anyhow::Result<Option<Item>> {
        let item = self.get_item_model(id).await?;

        let enriched = if let Some(item) = item.clone() {
            let enriched = self.enrich_item(item.clone()).await?;
            item.delete(&self.db).await?;

            Some(enriched)
        } else {
            None
        };

        Ok(enriched)
    }

    async fn update_item(
        &self,
        id: Uuid,
        name: Option<String>,
        url: Option<String>,
        body: Option<String>,
        summary: Option<String>,
        comment: Option<String>,
    ) -> anyhow::Result<Option<Item>> {
        let item = self.get_item_model(id).await?;

        let mut item: item::ActiveModel = if let Some(item) = item {
            item.into()
        } else {
            return Ok(None);
        };

        if let Some(name) = name {
            item.name = Set(name);
        }

        if let Some(url) = url {
            item.url = Set(Some(url));
        }

        if let Some(body) = body {
            item.body = Set(Some(body));
        }

        if let Some(summary) = summary {
            item.summary = Set(Some(summary));
        }

        if let Some(comment) = comment {
            item.comment = Set(Some(comment));
        }

        let item = item.update(&self.db).await?;

        Ok(Some(self.enrich_item(item).await?))
    }

    async fn item_tags(&self, id: Uuid) -> anyhow::Result<Vec<Tag>> {
        let item = if let Some(item) = self.get_item_model(id).await? {
            item
        } else {
            return Ok(vec![]);
        };

        let tags = item
            .find_related(tag::Entity)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(tags)
    }

    async fn linked_items(&self, item: Uuid) -> anyhow::Result<Vec<String>> {
        if let Some(item) = item::Entity::find_by_id(item).one(&self.db).await? {
            let related = item
                .find_linked(item::LinkedItem)
                .all(&self.db)
                .await?
                .into_iter()
                .map(|model| model.id.to_string())
                .collect();

            Ok(related)
        } else {
            Ok(vec![])
        }
    }

    async fn create_link(&self, a: Uuid, b: Uuid) -> anyhow::Result<()> {
        link::ActiveModel {
            a_id: Set(a),
            b_id: Set(b),
        }
        .insert(&self.db)
        .await?;

        Ok(())
    }

    async fn delete_link(&self, a: Uuid, b: Uuid) -> anyhow::Result<()> {
        if let Some(link) = link::Entity::find_by_id((a, b)).one(&self.db).await? {
            link.delete(&self.db).await?;
        }

        Ok(())
    }

    async fn blob(&self, id: Uuid) -> anyhow::Result<Option<Blob>> {
        self.get_blob_model(id).await.map(|o| o.map(Into::into))
    }

    async fn item_blob(&self, item: Uuid) -> anyhow::Result<Option<Blob>> {
        let item = if let Some(item) = self.get_item_model(item).await? {
            item
        } else {
            return Ok(None);
        };

        let blob = item.find_related(blob::Entity).one(&self.db).await?;

        Ok(blob.map(Into::into))
    }

    async fn add_blob(
        &self,
        item_id: Uuid,
        uri: String,
        hash: String,
        content_type: String,
        path: String,
        managed: bool,
    ) -> anyhow::Result<Blob> {
        blob::ActiveModel {
            id: Set(Uuid::new_v4()),
            item_id: Set(item_id),
            uri: Set(uri),
            hash: Set(hash),
            content_type: Set(content_type),
            path: Set(path),
            managed: Set(managed),
            created_at: Set(Utc::now()),
        }
        .insert(&self.db)
        .await
        .map(Into::into)
        .context("inserting a blob")
    }

    async fn delete_blob(&self, id: Uuid) -> anyhow::Result<Option<Blob>> {
        let blob = self.get_blob_model(id).await?;

        if let Some(blob) = blob.clone() {
            if blob.managed {
                tokio::fs::remove_file(&blob.path).await?;
            }

            blob.delete(&self.db).await?;
        }

        Ok(blob.map(Into::into))
    }
}

impl From<tag::Model> for Tag {
    fn from(model: tag::Model) -> Self {
        Tag {
            id: model.id,
            tag: model.tag,
            created_at: model.created_at,
        }
    }
}

impl From<blob::Model> for Blob {
    fn from(model: blob::Model) -> Self {
        Blob {
            id: model.id,
            uri: model.uri,
            hash: model.hash,
            content_type: model.content_type,
            path: model.path,
            managed: model.managed,
            created_at: model.created_at,
        }
    }
}
