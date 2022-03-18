use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Blob, Tag};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub url: Option<String>,
    pub body: Option<String>,
    pub comment: Option<String>,
    pub summary: Option<String>,
    pub tags: Vec<Tag>,
    pub blob: Option<Blob>,
    pub created_at: DateTime<Utc>,
}
