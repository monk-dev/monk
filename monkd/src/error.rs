use config::ConfigError;
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;
use url::ParseError;

use crate::metadata::Meta;

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
    #[error("Meta with ID already exists: {0}")]
    AlreadyExists(String),
    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Attempted to update with unequal IDs")]
    UnequalIds,
    #[error("ID Not found: `{0}`")]
    IdNotFound(String),
    #[error("Too many ids for: `{0}`, {1:?}")]
    TooManyIds(String, Vec<usize>),
    #[error("Too many ids for: `{0}`, {1:?}")]
    TooManyMetas(String, Vec<Meta>),
    #[error("Tantivy Error: {0}")]
    Tantivy(String),
    #[error("No url for: `{0}`")]
    NoUrl(String),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Error reading utf-8 data: {0}")]
    Utf8Conversion(#[from] std::string::FromUtf8Error),
    #[error("Tokio join error: {0}")]
    JoinHandle(#[from] tokio::task::JoinError),
    #[error("Custom: {0}")]
    Custom(String),
}

impl Error {
    pub fn is_client_error(&self) -> bool {
        match self {
            Error::IdNotFound(_)
            | Error::AlreadyExists(_)
            | Error::TooManyIds(_, _)
            | Error::TooManyMetas(_, _)
            | Error::NoUrl(_) => true,
            _ => false,
        }
    }
}
