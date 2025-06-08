use crate::{
    AddItem, Blob, CreateLink, DeleteItem, DeleteLink, EditItem, GetBlob, GetItem, Item,
    LinkedItems, ListItem, Search, SearchResult,
};

/// High-level interface for the Monk knowledge management system.
/// 
/// MonkTrait provides the main API for interacting with Monk, orchestrating
/// operations across storage, indexing, downloading, and extraction components.
/// This trait represents the complete functionality available to end users
/// and applications.
/// 
/// # Core Operations
/// - Item management (add, edit, delete, retrieve)
/// - Content downloading and extraction
/// - Full-text search
/// - Item linking and relationships
/// - Blob management for downloaded content
#[async_trait::async_trait]
pub trait MonkTrait {
    /// Adds a new item to the knowledge base.
    /// 
    /// Creates a new item with the provided metadata. Optionally triggers
    /// automatic downloading, content extraction, indexing, and summarization
    /// based on configuration settings.
    async fn add(&mut self, add: AddItem) -> anyhow::Result<Item>;

    /// Retrieves a single item by its ID.
    /// 
    /// Returns the complete item including metadata, tags, and associated blob
    /// information if available.
    async fn get(&mut self, get: GetItem) -> anyhow::Result<Option<Item>>;

    /// Retrieves a blob by item ID or blob ID.
    /// 
    /// Returns blob metadata and file information for downloaded content.
    async fn get_blob(&mut self, get: GetBlob) -> anyhow::Result<Option<Blob>>;

    /// Lists items from the knowledge base.
    /// 
    /// Returns a collection of items with optional count limiting.
    /// Items include full metadata, tags, and blob associations.
    async fn list(&mut self, list: ListItem) -> anyhow::Result<Vec<Item>>;

    /// Updates an existing item's metadata.
    /// 
    /// Modifies item properties such as name, URL, comment, body, or summary.
    /// Only provided fields are updated.
    async fn edit(&mut self, edit: EditItem) -> anyhow::Result<Option<Item>>;

    /// Removes an item from the knowledge base.
    /// 
    /// Deletes the item from storage and removes it from the search index.
    /// Associated blobs and links are also cleaned up.
    async fn delete(&mut self, delete: DeleteItem) -> anyhow::Result<Option<Item>>;

    /// Retrieves all items linked to the specified item.
    /// 
    /// Returns the IDs of items that have bidirectional links with the given item.
    async fn linked_items(&mut self, item: LinkedItems) -> anyhow::Result<Vec<String>>;

    /// Creates a bidirectional link between two items.
    /// 
    /// Establishes a relationship connection that can be traversed in both directions.
    async fn link(&mut self, link: CreateLink) -> anyhow::Result<()>;

    /// Removes a bidirectional link between two items.
    /// 
    /// Deletes the relationship connection between the specified items.
    async fn unlink(&mut self, link: DeleteLink) -> anyhow::Result<()>;

    /// Performs a full-text search across the knowledge base.
    /// 
    /// Searches indexed content including item names, extracted text, and comments.
    /// Returns ranked results with snippets and highlighting information.
    async fn search(&mut self, search: Search) -> anyhow::Result<Vec<SearchResult>>;
    // async fn status(&self, status: Status);
}
