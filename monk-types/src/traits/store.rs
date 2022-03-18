use uuid::Uuid;

use crate::{Blob, Item, Tag};

#[async_trait::async_trait]
pub trait Store {
    async fn item(&self, id: Uuid) -> anyhow::Result<Option<Item>>;

    async fn list_items(&self) -> anyhow::Result<Vec<Item>>;

    async fn add_item(
        &self,
        name: String,
        url: Option<String>,
        comment: Option<String>,
        tags: Vec<String>,
    ) -> anyhow::Result<Item>;

    async fn delete_item(&self, id: Uuid) -> anyhow::Result<Option<Item>>;

    async fn update_item(
        &self,
        id: Uuid,
        name: Option<String>,
        url: Option<String>,
        body: Option<String>,
        summary: Option<String>,
        comment: Option<String>,
    ) -> anyhow::Result<Option<Item>>;

    async fn item_tags(&self, item: Uuid) -> anyhow::Result<Vec<Tag>>;

    async fn linked_items(&self, item: Uuid) -> anyhow::Result<Vec<String>>;

    async fn create_link(&self, a: Uuid, b: Uuid) -> anyhow::Result<()>;

    async fn delete_link(&self, a: Uuid, b: Uuid) -> anyhow::Result<()>;

    async fn item_blob(&self, item: Uuid) -> anyhow::Result<Option<Blob>>;

    async fn blob(&self, id: Uuid) -> anyhow::Result<Option<Blob>>;

    async fn add_blob(
        &self,
        item_id: Uuid,
        uri: String,
        hash: String,
        content_type: String,
        path: String,
        managed: bool,
    ) -> anyhow::Result<Blob>;

    async fn delete_blob(&self, id: Uuid) -> anyhow::Result<Option<Blob>>;
}
