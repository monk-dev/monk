use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tantivy::Snippet;

use crate::error::Error;
use crate::metadata::offline_store::Status as OfflineStatus;
use crate::metadata::{meta::IndexStatus, Meta};
use crate::status::StatusResponse;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Response {
    NewId(String),
    Item(Meta),
    List(Vec<Meta>),
    Error(String),
    Custom(String),
    NoAdapterFound(String),
    NotFound(String),
    TooManyMeta(String, Vec<Meta>),
    MetaOfflineStatus(String, OfflineStatus),
    OpenStatus(String, OfflineStatus),
    IndexStatus(String, Option<IndexStatus>),
    Indexing(String),
    Status(StatusResponse),
    SearchResult(Vec<(Meta, SnippetDef)>), // Meta, Fragment, Highlight
    Many(Vec<Response>),
    Open(PathBuf),
    Unhandled,
    Ok,
}

impl From<Error> for Response {
    fn from(e: Error) -> Self {
        match e {
            Error::AlreadyExists(id) => Response::Error(format!("`{}` already exists", id)),
            Error::TooManyMetas(id, metas) => Response::TooManyMeta(id, metas),
            e => Response::Error(e.to_string()),
        }
    }
}

// Bellow is a shameless copy of tantivy's snippet class. This was done so that
// I could use serde.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HighlightSectionDef {
    start: usize,
    stop: usize,
}
impl HighlightSectionDef {
    pub fn bounds(&self) -> (usize, usize) {
        (self.start, self.stop)
    }
}

// geto work around for Serde to work on tantivy
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SnippetDef {
    fragment: String,
    highlighted: Vec<HighlightSectionDef>,
}
impl SnippetDef {
    pub fn highlighted(&self) -> &[HighlightSectionDef] {
        &self.highlighted
    }
    pub fn fragment(&self) -> &str {
        &self.fragment
    }
}

impl From<Snippet> for SnippetDef {
    fn from(item: Snippet) -> Self {
        let mut sections: Vec<HighlightSectionDef> = Vec::new();
        for (start, stop) in item.highlighted().iter().map(|h| h.bounds()) {
            let highlight = HighlightSectionDef { start, stop };
            sections.push(highlight);
        }
        SnippetDef {
            fragment: item.fragments().to_string(),
            highlighted: sections,
        }
    }
}
