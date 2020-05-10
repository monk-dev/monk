use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Meta {
    name: String,
    url: Url,
    // res: Option<PathBuf>,
    #[serde(with = "ts_milliseconds")]
    found: DateTime<Utc>,
    last_read: Option<DateTime<Utc>>,
}

impl Meta {
    pub fn new(name: impl Into<String>, url: &str, found: DateTime<Utc>) -> Result<Self, Error> {
        Ok(Self {
            name: name.into(),
            url: Url::parse(url)?,
            found,
            last_read: None,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn found(&self) -> &DateTime<Utc> {
        &self.found
    }
}
