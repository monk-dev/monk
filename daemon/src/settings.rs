use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::{Path, PathBuf};

/// The main Daemon settings. Defaults are:
/// address: 127.0.0.1:8888
/// timeout: 1000
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    address: IpAddr,
    port: u16,
    timeout: usize,
    #[serde(default = "default_path")]
    store: PathBuf,
}

fn default_path() -> PathBuf {
    PathBuf::from("./store.json")
}

impl Settings {
    pub fn address(&self) -> &IpAddr {
        &self.address
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn timeout(&self) -> usize {
        self.timeout
    }

    pub fn store(&self) -> &Path {
        &self.store
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".parse().unwrap(),
            port: 8888,
            timeout: 1000,
            store: default_path(),
        }
    }
}