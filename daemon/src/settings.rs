use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// The main Daemon settings. Defaults are:
/// address: 127.0.0.1:8888
/// timeout: 1000
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    address: IpAddr,
    timeout: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:8888".parse().unwrap(),
            timeout: 1000,
        }
    }
}

// impl Settings {

// }
