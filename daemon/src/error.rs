use thiserror::Error;

use url::ParseError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Url Parsing Error: {0}")]
    UrlParse(#[from] ParseError),
    #[error("Tokio IO Error: {0}")]
    TokioIO(#[from] tokio::io::Error),
}
