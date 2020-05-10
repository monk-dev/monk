use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
    Add { name: String, url: Url },
    List { count: usize },
    Stop,
}
