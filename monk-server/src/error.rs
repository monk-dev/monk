use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Sql: {0}")]
    Sql(#[from] rusqlite::Error),
}
