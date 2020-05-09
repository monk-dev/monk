use failure::{Compat, Fail};
use thiserror::Error;

use tantivy::{directory::error::OpenDirectoryError, TantivyError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Tantivy Error: {}", 0)]
    Tantivy(#[from] Compat<TantivyError>),
    #[error("Tantivy Error: {}", 0)]
    TantivyOpenDirectory(#[from] Compat<OpenDirectoryError>),
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
