use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a categorization label for organizing items.
/// 
/// Tags provide a flexible way to categorize, group, and filter items
/// in the knowledge base. They enable users to organize content by
/// topics, projects, importance, or any other classification scheme.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    /// Unique identifier for this tag.
    pub id: Uuid,
    
    /// The tag label or name.
    /// A descriptive string used for categorization and display.
    pub tag: String,
    
    /// Timestamp when the tag was first created.
    /// Tracks tag creation for historical purposes.
    pub created_at: DateTime<Utc>,
}
