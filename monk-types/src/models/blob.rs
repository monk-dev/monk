use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Blob {
    pub id: Uuid,
    pub uri: String,
    pub hash: String,
    pub content_type: String,
    pub path: String,
    pub managed: bool,
    pub created_at: DateTime<Utc>,
}
