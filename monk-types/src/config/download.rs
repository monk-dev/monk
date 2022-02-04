use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub path: PathBuf,
    pub download_on_add: bool,
}
