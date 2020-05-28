use serde::{Deserialize, Serialize};
use url::Url;

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
        id: String,
    },
    Open {
        id: String,
    },
    ForceShutdown,
    Stop,
}
