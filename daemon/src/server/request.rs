use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
    Add { name: String, url: Option<Url> },
    List { count: Option<usize> },
    Delete { id: String },
    Get { id: String },
    ForceShutdown,
    Stop,
}
