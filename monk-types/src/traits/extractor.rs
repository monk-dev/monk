use crate::{Blob, ExtractedInfo, Item};

/// Extracts textual content and metadata from downloaded blobs.
/// 
/// The Extractor trait processes downloaded content to extract searchable text,
/// metadata, and other structured information. It handles various content types
/// including HTML, PDF, and other document formats.
/// 
/// # Purpose
/// - Convert binary/formatted content into searchable text
/// - Extract metadata and structured information
/// - Prepare content for indexing and search
/// - Enable content-based search and analysis
#[async_trait::async_trait]
pub trait Extractor {
    /// Extracts textual content and metadata from an item's blob.
    /// 
    /// Processes the downloaded content (if available) to extract searchable text,
    /// titles, metadata, and other structured information. Returns None if no
    /// extractable content is found or if extraction fails.
    /// 
    /// The extracted information is used for full-text indexing and search.
    async fn extract_info(
        &self,
        item: &Item,
        blob: Option<&Blob>,
    ) -> anyhow::Result<Option<ExtractedInfo>>;
}
