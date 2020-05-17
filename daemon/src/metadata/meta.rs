use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use std::path::PathBuf;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Meta {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) url: Option<Url>,
    // res: Option<PathBuf>,
    #[serde(with = "ts_milliseconds")]
    pub(crate) found: DateTime<Utc>,
    pub(crate) last_read: Option<DateTime<Utc>>,
}

impl Meta {
    // pub fn new(name: impl Into<String>, url: Url, found: DateTime<Utc>) -> Self {
    //     Self {
    //         id,
    //         name: name.into(),
    //         url,
    //         found,
    //         last_read: None,
    //     }
    // }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    pub fn found(&self) -> &DateTime<Utc> {
        &self.found
    }

    pub fn builder(name: impl Into<String>) -> MetaBuilder {
        MetaBuilder::new(name)
    }
}

pub struct MetaBuilder {
    id: Option<String>,
    name: String,
    url: Option<Url>,
    found: Option<DateTime<Utc>>,
    last_read: Option<DateTime<Utc>>,
}

impl MetaBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        MetaBuilder {
            name: name.into(),
            url: None,
            id: None,
            found: None,
            last_read: None,
        }
    }

    pub fn id(self, id: String) -> Self {
        Self {
            id: Some(id),
            ..self
        }
    }

    pub fn found(self, found: DateTime<Utc>) -> Self {
        Self {
            found: Some(found),
            ..self
        }
    }

    pub fn last_read(self, last_read: DateTime<Utc>) -> Self {
        Self {
            last_read: Some(last_read),
            ..self
        }
    }

    pub fn url(self, url: Url) -> Self {
        Self {
            url: Some(url),
            ..self
        }
    }

    pub fn build(self) -> Meta {
        let found = if let Some(found) = self.found {
            found
        } else {
            Utc::now()
        };

        let id = if let Some(id) = self.id {
            id
        } else {
            crate::generate_id()
        };

        Meta {
            id,
            name: self.name,
            url: self.url,
            found,
            last_read: self.last_read,
        }
    }
}
