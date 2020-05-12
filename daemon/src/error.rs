use thiserror::Error;

use config::ConfigError;
use tokio::sync::oneshot::error::RecvError;
use url::ParseError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Url Parsing Error: {0}")]
    UrlParse(#[from] ParseError),
    #[error("Tokio IO Error: {0}")]
    TokioIO(#[from] tokio::io::Error),
    #[error("Metastore has no path")]
    FileStoreNoPath,
    #[error("Configuration Parse Error: {0}")]
    ConfigError(#[from] ConfigError),
    #[error("Tokio oneshot Recv Error: {0}")]
    RecvError(#[from] RecvError),
    #[error("Custom: {0}")]
    Custom(String),
}
