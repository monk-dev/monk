use serde::{Deserialize, Serialize};
use url::Url;

use crate::metadata::{offline_store::OfflineData, Meta};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
    Add {
        name: Option<String>,
        url: Option<Url>,
        comment: Option<String>,
        tags: Vec<String>,
    },
    List {
        count: Option<usize>,
        tags: Vec<String>,
    },
    Edit {
        id: String,
        edit: Edit,
    },
    Delete {
        id: String,
    },
    Get {
        id: String,
    },
    Download {
        id: Option<String>,
    },
    Open {
        id: String,
        online: bool,
    },
    Search {
        count: Option<usize>,
        query: String,
    },
    Index {
        id: String,
    },
    IndexAll {
        tags: Vec<String>,
    },
    IndexStatus {
        id: String,
    },
    Status {
        kind: StatusKind,
    },
    ForceShutdown,
    Stop,
    #[serde(skip)]
    UpdateOffline(OfflineData),
    #[serde(skip)]
    UpdateMeta(Meta),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusKind {
    All,
    Index,
    Store,
    Offline,
    Id(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit {
    pub name: Option<String>,
    pub url: Option<String>,
    pub comment: Option<String>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
}
