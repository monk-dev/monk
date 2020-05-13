use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
    Add { name: String, url: Url },
    List { count: Option<usize> },
    Stop,
}
