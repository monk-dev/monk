use uuid::Uuid;

use crate::models::index::ExtractedInfo;
use crate::{Item, SearchResult};

/// Full-text search and indexing engine for Monk's knowledge base.
/// 
/// The Index trait provides search capabilities across all stored content,
/// including item metadata, extracted text, and summaries. It manages the
/// search index lifecycle from content ingestion to query processing.
/// 
/// # Features
/// - Full-text search with relevance scoring
/// - Snippet extraction and highlighting
/// - Multi-field search (name, content, comments)
/// - Index management and optimization
pub trait Index {
    /// Returns the total number of indexed items.
    fn count(&self) -> anyhow::Result<usize>;

    /// Performs a full-text search and returns ranked results.
    /// 
    /// Searches across all indexed content including item names, extracted text,
    /// and comments. Returns results ranked by relevance with snippets and
    /// highlighting information for display.
    fn search(&self, query: &str, count: usize) -> anyhow::Result<Vec<SearchResult>>;

    /// Indexes an item using only its basic metadata.
    /// 
    /// Default implementation that indexes the item without additional extracted content.
    /// Equivalent to calling `index_full` with empty ExtractedInfo.
    fn index(&mut self, item: &Item) -> anyhow::Result<()> {
        self.index_full(item, Default::default())
    }

    /// Indexes an item with extracted content and metadata.
    /// 
    /// Adds or updates the item in the search index, including any extracted
    /// text content, metadata, and other searchable information. This is the
    /// primary method for making content searchable.
    fn index_full(&mut self, item: &Item, extra: ExtractedInfo) -> anyhow::Result<()>;

    /// Generates a summary of the provided text.
    /// 
    /// Creates a concise summary of longer text content for storage and display.
    /// Used to generate item summaries from extracted content.
    fn summarize(&mut self, text: &str) -> anyhow::Result<String>;

    /// Removes an item from the search index.
    /// 
    /// Deletes all indexed content for the specified item, making it no longer
    /// searchable. Used when items are deleted or need reindexing.
    fn remove(&mut self, id: Uuid) -> anyhow::Result<()>;
}
