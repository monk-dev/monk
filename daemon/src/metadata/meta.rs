use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use std::path::PathBuf;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Meta {
    id: String,
    name: String,
    url: Url,
    // res: Option<PathBuf>,
    #[serde(with = "ts_milliseconds")]
    found: DateTime<Utc>,
    last_read: Option<DateTime<Utc>>,
}

impl Meta {
    pub fn new(name: impl Into<String>, url: &str, found: DateTime<Utc>) -> Result<Self, Error> {
        let id = crate::generate_id();

        Ok(Self {
            id,
            name: name.into(),
            url: Url::parse(url)?,
            found,
            last_read: None,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
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
