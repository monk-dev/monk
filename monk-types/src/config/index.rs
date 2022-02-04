use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexConfig {
    pub path: PathBuf,
    pub index_on_add: bool,
}
