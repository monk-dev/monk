use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedInfo {
    pub title: Option<String>,
    pub body: Option<String>,
    pub extra: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub score: f32,
    pub snippets: Snippets,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snippets {
    pub title: Snippet,
    pub body: Snippet,
    pub comment: Snippet,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snippet {
    pub fragment: String,
    pub highlighted: Vec<(usize, usize)>,
}
