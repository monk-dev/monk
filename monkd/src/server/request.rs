use serde::{Deserialize, Serialize};
use url::Url;

use crate::metadata::{offline_store::OfflineData, Meta};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
    Add {
        name: Option<String>,
        url: Option<Url>,
        comment: Option<String>,
    },
    List {
        count: Option<usize>,
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
    },
    Search {
        query: String,
    },
    Index {
        id: String,
    },
    IndexAll,
    IndexStatus,
    ForceShutdown,
    Stop,
    #[serde(skip)]
    UpdateOffline(OfflineData),
    #[serde(skip)]
    UpdateMeta(Meta),
}
