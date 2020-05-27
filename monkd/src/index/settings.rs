use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexSettings {
    pub(crate) path: PathBuf,
    // commit_interval: usize
}

impl Default for IndexSettings {
    fn default() -> Self {
        Self {
            path: "./index".into(),
        }
    }
}
