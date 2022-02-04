use crate::{
    AddItem, Blob, CreateLink, DeleteItem, DeleteLink, EditItem, GetBlob, GetItem, Item,
    LinkedItems, ListItem, Search, SearchResult,
};

#[async_trait::async_trait]
pub trait MonkTrait {
    async fn add(&mut self, add: AddItem) -> anyhow::Result<Item>;

    async fn get(&mut self, get: GetItem) -> anyhow::Result<Option<Item>>;

    async fn get_blob(&mut self, get: GetBlob) -> anyhow::Result<Option<Blob>>;

    async fn list(&mut self, list: ListItem) -> anyhow::Result<Vec<Item>>;

    async fn edit(&mut self, edit: EditItem) -> anyhow::Result<Option<Item>>;

    async fn delete(&mut self, delete: DeleteItem) -> anyhow::Result<Option<Item>>;

    async fn linked_items(&mut self, item: LinkedItems) -> anyhow::Result<Vec<String>>;

    async fn link(&mut self, link: CreateLink) -> anyhow::Result<()>;

    async fn unlink(&mut self, link: DeleteLink) -> anyhow::Result<()>;

    async fn search(&mut self, search: Search) -> anyhow::Result<Vec<SearchResult>>;
    // async fn status(&self, status: Status);
}
