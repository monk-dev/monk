use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexSettings {
    pub(crate) path: PathBuf,
    // commit_interval: usize
}

impl Default for IndexSettings {
    fn default() -> Self {
        if let Some(dirs) = crate::get_dirs() {
            Self {
                path: dirs.data_dir().join("index"),
            }
        } else {
            Self {
                path: "./index".into(),
            }
        }
    }
}
