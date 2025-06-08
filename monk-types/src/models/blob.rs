use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents downloaded content and file metadata for items.
/// 
/// A Blob stores information about files that have been downloaded and
/// saved locally by the Monk system. This includes web pages, documents,
/// images, and other content types that can be downloaded from URLs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Blob {
    /// Unique identifier for this blob.
    pub id: Uuid,
    
    /// Original URI where the content was downloaded from.
    /// Preserves the source location for reference and re-downloading.
    pub uri: String,
    
    /// Content hash for integrity verification.
    /// Used to detect changes in content and avoid duplicate downloads.
    pub hash: String,
    
    /// MIME type or content type of the downloaded file.
    /// Helps determine how to process and display the content.
    pub content_type: String,
    
    /// Local file system path where the content is stored.
    /// Points to the actual downloaded file on disk.
    pub path: String,
    
    /// Whether this blob's file is managed by Monk.
    /// If true, Monk will handle cleanup and lifecycle management.
    /// If false, the file is externally managed.
    pub managed: bool,
    
    /// Timestamp when the blob was created.
    /// Tracks when the content was originally downloaded.
    pub created_at: DateTime<Utc>,
}
