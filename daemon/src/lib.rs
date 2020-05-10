pub mod config;
pub mod error;
pub mod metadata;
pub mod request;
pub mod server;

use crate::config::Config;
use anyhow::Result;

pub async fn run(config: &Config) -> Result<()> {
    Ok(())
}
