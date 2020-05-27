use serde::{Deserialize, Serialize};
use std::net::IpAddr;

// /// The main Daemon settings. Defaults are:
// /// address: 127.0.0.1:8888
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Settings {
//     address: IpAddr,
//     port: u16,
//     // timeout: usize,
// }

// impl Settings {
//     pub fn address(&self) -> &IpAddr {
//         &self.address
//     }

//     pub fn port(&self) -> u16 {
//         self.port
//     }
// }

// impl Default for Settings {
//     fn default() -> Self {
//         Self {
//             address: "127.0.0.1".parse().unwrap(),
//             port: 8888,
//         }
//     }
// }
