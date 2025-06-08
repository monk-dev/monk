use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Information extracted from downloaded content for indexing and search.
/// 
/// Contains textual content and metadata extracted from various file formats
/// including HTML, PDF, and other document types. This extracted information
/// is used for full-text indexing and search capabilities.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedInfo {
    /// Extracted title or heading from the content.
    /// Typically the main title, document title, or page title.
    pub title: Option<String>,
    
    /// Main textual content extracted from the source.
    /// Contains the primary readable text for indexing and search.
    pub body: Option<String>,
    
    /// Additional extracted information or metadata.
    /// May contain supplementary text, metadata, or other relevant content.
    pub extra: Option<String>,
}

/// Represents a single search result with relevance scoring and snippets.
/// 
/// Contains the item ID, relevance score, and contextual snippets showing
/// where search terms were found within the item's content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    /// ID of the item that matched the search query.
    pub id: Uuid,
    
    /// Relevance score for this search result.
    /// Higher scores indicate better matches to the search query.
    pub score: f32,
    
    /// Collection of text snippets showing search term matches.
    /// Provides context for where and how the search terms were found.
    pub snippets: Snippets,
}

/// Collection of text snippets from different fields of an item.
/// 
/// Provides contextual excerpts from various parts of an item where
/// search terms were found, with highlighting information for display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snippets {
    /// Snippet from the item's name or title field.
    pub name: Snippet,
    
    /// Snippet from the item's body content.
    pub body: Snippet,
    
    /// Snippet from the item's comment field.
    pub comment: Snippet,
}

/// A text fragment with search term highlighting information.
/// 
/// Contains a portion of text and the character ranges where search
/// terms should be highlighted for display purposes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snippet {
    /// The text fragment containing search terms.
    pub fragment: String,
    
    /// Character ranges to highlight within the fragment.
    /// Each tuple represents (start_index, end_index) for highlighting.
    pub highlighted: Vec<(usize, usize)>,
}
