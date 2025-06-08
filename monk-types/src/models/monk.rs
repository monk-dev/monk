use serde::{Deserialize, Serialize};

/// Request to add a new item to the knowledge base.
/// 
/// Contains all the metadata needed to create a new item, including
/// optional content and categorization information.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddItem {
    /// Name or title for the new item.
    pub name: String,
    
    /// Optional URL where the content can be found.
    pub url: Option<String>,
    
    /// Optional body content for the item.
    pub body: Option<String>,
    
    /// Optional user comment or description.
    pub comment: Option<String>,
    
    /// List of tag names to associate with the item.
    pub tags: Vec<String>,
}

/// Request to retrieve a specific item by ID.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetItem {
    /// String representation of the item's UUID.
    pub id: String,
}

/// Request to retrieve tag information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GetTag {
    /// Get tags by item ID.
    ItemId(String),
    /// Get a specific tag by tag ID.
    TagId(String),
}

/// Request to retrieve blob information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GetBlob {
    /// Get blob by associated item ID.
    ItemId(String),
    /// Get blob by specific blob ID.
    BlobId(String),
}

/// Request to list items with optional filtering.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    /// Maximum number of items to return.
    pub count: Option<usize>,
    
    /// Filter items by these tag names.
    pub tags: Vec<String>,
}

/// Request to modify an existing item.
/// 
/// For all optional fields, only provided fields will be updated.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditItem {
    /// ID of the item to edit.
    pub id: String,
    
    /// New name for the item.
    pub name: Option<String>,
    
    /// New URL for the item.
    pub url: Option<String>,
    
    /// New body content for the item.
    pub body: Option<String>,
    
    /// New summary for the item.
    pub summary: Option<String>,
    
    /// New comment for the item.
    pub comment: Option<String>,
    
    /// Tag names to add to the item.
    pub add_tags: Vec<String>,
    
    /// Tag names to remove from the item.
    pub remove_tags: Vec<String>,
}

/// Request to delete an item.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteItem {
    /// ID of the item to delete.
    pub id: String,
}

/// Request to get items linked to a specific item.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkedItems {
    /// ID of the item to find links for.
    pub id: String,
}

/// Request to create a link between two items.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateLink {
    /// ID of the first item.
    pub a: String,
    
    /// ID of the second item.
    pub b: String,
}

/// Request to delete a link between two items.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteLink {
    /// ID of the first item.
    pub a: String,
    
    /// ID of the second item.
    pub b: String,
}

/// Request to search the knowledge base.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Search {
    /// Maximum number of results to return.
    pub count: Option<usize>,
    
    /// Search query string.
    pub query: String,
}

/// Request for system status information.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// Optional item ID to get status for.
    pub id: Option<String>,
    
    /// Whether to include index status.
    pub index: bool,
    
    /// Whether to include store status.
    pub store: bool,
    
    /// Whether to include offline status.
    pub offline: bool,
}
