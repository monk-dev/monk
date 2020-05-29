use failure::{Compat, Fail};
use thiserror::Error;

use tantivy::{directory::error::OpenDirectoryError, TantivyError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Tantivy: {0}")]
    Tantivy(#[from] Compat<TantivyError>),
    #[error("Tantivy: {0}")]
    TantivyOpenDirectory(#[from] Compat<OpenDirectoryError>),
    #[error("Configuration Parse: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Url Parsing: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Reqwest: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}

impl From<TantivyError> for Error {
    fn from(e: TantivyError) -> Self {
        Self::Tantivy(e.compat())
    }
}

impl From<OpenDirectoryError> for Error {
    fn from(e: OpenDirectoryError) -> Self {
        Self::TantivyOpenDirectory(e.compat())
    }
}
