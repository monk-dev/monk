use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use crate::index::settings::IndexSettings;
use crate::metadata::{file_store::StoreSettings, offline_store::OfflineSettings};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    log_dir: PathBuf,
    daemon: DaemonSettings,
    store: StoreSettings,
    offline: OfflineSettings,
    index: IndexSettings,
}

impl Settings {
    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    pub fn daemon(&self) -> &DaemonSettings {
        &self.daemon
    }
    pub fn store(&self) -> &StoreSettings {
        &self.store
    }
    pub fn offline(&self) -> &OfflineSettings {
        &self.offline
    }

    pub fn index(&self) -> &IndexSettings {
        &self.index
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            log_dir: "/var/log/monk".into(),
            ..Default::default()
        }
    }
}

/// The main Daemon settings. Defaults are:
/// address: 127.0.0.1:8888
/// timeout: 1000
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSettings {
    pub address: IpAddr,
    pub port: u16,
    pub timeout: usize,
}

// fn default_store() -> PathBuf {
//     PathBuf::from("./store.json")
// }

// fn default_offline() -> PathBuf {
//     PathBuf::from("./offline.json")
// }

impl Default for DaemonSettings {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".parse().unwrap(),
            port: 8888,
            timeout: 1000,
        }
    }
}
