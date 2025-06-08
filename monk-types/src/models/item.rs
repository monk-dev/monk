use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Blob, Tag};

/// Core data model representing a knowledge item in the Monk system.
/// 
/// An Item is the fundamental unit of information storage, representing
/// any piece of knowledge that a user wants to save, organize, and search.
/// Items can represent web pages, documents, notes, or any other content
/// with associated metadata.
/// 
/// # Key Features
/// - Unique identification with UUID
/// - Rich metadata including name, URL, and comments
/// - Extracted and summarized content for search
/// - Tag-based categorization and organization
/// - Optional associated blob for downloaded content
/// - Timestamp tracking for creation time
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// Unique identifier for the item across the system.
    pub id: Uuid,
    
    /// Human-readable name or title for the item.
    /// This is the primary display name shown in lists and search results.
    pub name: String,
    
    /// Optional URL source where the content originated.
    /// Used for web content, documents, or any network-accessible resource.
    pub url: Option<String>,
    
    /// Extracted textual content from the source.
    /// Contains the full text content extracted from downloaded files,
    /// web pages, or manually entered text for search indexing.
    pub body: Option<String>,
    
    /// User-provided comment or description.
    /// Personal notes, context, or additional information about the item.
    pub comment: Option<String>,
    
    /// Concise summary of the item's content.
    /// A condensed version of the body text for quick overview.
    pub summary: Option<String>,
    
    /// Collection of tags for categorization and filtering.
    /// Tags provide a flexible way to organize and group related items.
    pub tags: Vec<Tag>,
    
    /// Optional associated blob containing downloaded content.
    /// References the actual file or content downloaded from the URL,
    /// including metadata about the file type and storage location.
    pub blob: Option<Blob>,
    
    /// Timestamp when the item was created.
    /// Used for sorting, filtering, and tracking item history.
    pub created_at: DateTime<Utc>,
}
