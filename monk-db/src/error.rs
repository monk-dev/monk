use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    SqlLite(#[from] rusqlite::Error),
    #[error("entity not found: {0}")]
    EntityNotFound(uuid::Uuid),
}
