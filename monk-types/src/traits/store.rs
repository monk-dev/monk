use uuid::Uuid;

use crate::{Blob, Item, Tag};

/// Core data persistence layer for the Monk knowledge management system.
/// 
/// The Store trait defines the interface for all data storage operations including
/// items, tags, links, and blobs. It provides CRUD operations for the main entities
/// and handles relationships between them.
/// 
/// # Architecture
/// - Items are the core entity representing saved knowledge pieces
/// - Tags provide categorization and metadata
/// - Links create relationships between items
/// - Blobs store downloaded content associated with items
#[async_trait::async_trait]
pub trait Store {
    /// Retrieves a single item by its unique identifier.
    /// Returns None if the item doesn't exist.
    async fn item(&self, id: Uuid) -> anyhow::Result<Option<Item>>;

    /// Retrieves all items from the store.
    /// Items include associated tags and blob information.
    async fn list_items(&self) -> anyhow::Result<Vec<Item>>;

    /// Creates a new item with the provided metadata.
    /// Tags are created automatically if they don't exist.
    /// Returns the newly created item with generated ID and timestamp.
    async fn add_item(
        &self,
        name: String,
        url: Option<String>,
        comment: Option<String>,
        tags: Vec<String>,
    ) -> anyhow::Result<Item>;

    /// Removes an item from the store.
    /// Returns the deleted item if it existed, None otherwise.
    async fn delete_item(&self, id: Uuid) -> anyhow::Result<Option<Item>>;

    /// Updates an existing item's metadata.
    /// Only provided fields are updated; None values are ignored.
    /// Returns the updated item if it existed, None otherwise.
    async fn update_item(
        &self,
        id: Uuid,
        name: Option<String>,
        url: Option<String>,
        body: Option<String>,
        summary: Option<String>,
        comment: Option<String>,
    ) -> anyhow::Result<Option<Item>>;

    /// Retrieves all tags associated with a specific item.
    async fn item_tags(&self, item: Uuid) -> anyhow::Result<Vec<Tag>>;

    /// Retrieves the IDs of all items linked to the specified item.
    /// Links are bidirectional relationships between items.
    async fn linked_items(&self, item: Uuid) -> anyhow::Result<Vec<String>>;

    /// Creates a bidirectional link between two items.
    /// Links represent relationships or connections between knowledge pieces.
    async fn create_link(&self, a: Uuid, b: Uuid) -> anyhow::Result<()>;

    /// Removes a bidirectional link between two items.
    async fn delete_link(&self, a: Uuid, b: Uuid) -> anyhow::Result<()>;

    /// Retrieves the blob (downloaded content) associated with an item.
    /// Returns None if the item has no associated blob.
    async fn item_blob(&self, item: Uuid) -> anyhow::Result<Option<Blob>>;

    /// Retrieves a blob by its unique identifier.
    /// Returns None if the blob doesn't exist.
    async fn blob(&self, id: Uuid) -> anyhow::Result<Option<Blob>>;

    /// Creates a new blob entry associated with an item.
    /// Stores metadata about downloaded content including file path and hash.
    /// The `managed` flag indicates whether the blob file is managed by Monk.
    async fn add_blob(
        &self,
        item_id: Uuid,
        uri: String,
        hash: String,
        content_type: String,
        path: String,
        managed: bool,
    ) -> anyhow::Result<Blob>;

    /// Removes a blob entry from the store.
    /// Returns the deleted blob if it existed, None otherwise.
    async fn delete_blob(&self, id: Uuid) -> anyhow::Result<Option<Blob>>;
}
