use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Meta {
    pub(crate) id: String,
    pub(crate) name: Option<String>,
    pub(crate) url: Option<Url>,
    pub(crate) comment: Option<String>,
    #[serde(with = "ts_milliseconds")]
    pub(crate) found: DateTime<Utc>,
    pub(crate) last_read: Option<DateTime<Utc>>,
    #[serde(default = "default_time")]
    pub(crate) last_updated: DateTime<Utc>,
    pub(crate) index_status: Option<IndexStatus>,
    #[serde(default)]
    pub(crate) tags: BTreeSet<String>,
}

impl Meta {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    pub fn found(&self) -> &DateTime<Utc> {
        &self.found
    }

    pub fn last_read(&self) -> Option<&DateTime<Utc>> {
        self.last_read.as_ref()
    }

    pub fn tags(&self) -> &BTreeSet<String> {
        &self.tags
    }

    pub fn builder() -> MetaBuilder {
        MetaBuilder::new()
    }

    pub fn union(&mut self, other: &Meta) {
        let newwest;
        newwest = self.last_updated >= other.last_updated;

        if !newwest {
            if let Some(_) = other.name {
                self.name = other.name.clone();
            }
            if let Some(_) = other.comment {
                self.comment = other.comment.clone();
            }
            if let Some(_) = other.last_read {
                self.last_read = other.last_read;
            }
            self.found = other.found;
        } else {
            if self.name == None {
                self.name = other.name.clone();
            }
            if self.comment == None {
                self.comment = other.comment.clone();
            }
            if self.last_read == None {
                self.last_read = other.last_read;
            }
        }

        self.tags = self.tags.union(&other.tags).cloned().collect();

        self.last_updated = Utc::now();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IndexStatus {
    Indexing,
    Indexed,
    Old,
}

impl IndexStatus {
    pub fn is_indexing(&self) -> bool {
        *self == IndexStatus::Indexing
    }

    pub fn is_indexed(&self) -> bool {
        *self == IndexStatus::Indexed
    }

    pub fn is_old(&self) -> bool {
        *self == IndexStatus::Old
    }
}

pub struct MetaBuilder {
    id: Option<String>,
    name: Option<String>,
    url: Option<Url>,
    comment: Option<String>,
    found: Option<DateTime<Utc>>,
    last_read: Option<DateTime<Utc>>,
    tags: Option<BTreeSet<String>>,
}

impl MetaBuilder {
    pub fn new() -> Self {
        MetaBuilder {
            name: None,
            url: None,
            id: None,
            comment: None,
            found: None,
            last_read: None,
            tags: None,
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

    pub fn name(self, name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    pub fn comment(self, comment: impl Into<String>) -> Self {
        Self {
            comment: Some(comment.into()),
            ..self
        }
    }

    pub fn tags(self, tags: BTreeSet<String>) -> Self {
        Self {
            tags: Some(tags),
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

        let tags = if let Some(tags) = self.tags {
            tags
        } else {
            BTreeSet::new()
        };

        Meta {
            id,
            name: self.name,
            url: self.url,
            comment: self.comment,
            found,
            last_read: self.last_read,
            last_updated: Utc::now(),
            index_status: None,
            tags,
        }
    }
}

impl Default for MetaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Meta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.id())?;

        if let Some(name) = self.name() {
            write!(f, " {}:", name)?;
        } else {
            write!(f, "n/a:")?;
        }

        if let Some(url) = self.url() {
            write!(f, " {}", url.to_string())?;
        }

        let found = self.found.format("%a %d, %Y").to_string();
        write!(f, " @ {}", found)?;

        if !self.tags().is_empty() {
            self.tags
                .iter()
                .for_each(|tag| write!(f, "\n\t{}", tag).unwrap_or(()));
        }

        if let Some(comment) = self.comment() {
            write!(f, "\n\t{}", comment)?;
        }

        Ok(())
    }
}

fn default_time() -> DateTime<Utc> {
    Utc::now()
}
