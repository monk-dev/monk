use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Error;
use crate::metadata::offline_store::Status as OfflineStatus;
use crate::metadata::{meta::IndexStatus, Meta};
use crate::status::StatusResponse;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Response {
    NewId(String),
    Item(Meta),
    List(Vec<Meta>),
    Error(String),
    NoAdapterFound(String),
    NotFound(String),
    TooManyMeta(String, Vec<Meta>),
    MetaOfflineStatus(String, OfflineStatus),
    OpenStatus(String, OfflineStatus),
    IndexStatus(String, Option<IndexStatus>),
    Indexing(String),
    Status(StatusResponse),
    Many(Vec<Response>),
    Open(PathBuf),
    Unhandled,
    Ok,
}

impl From<Error> for Response {
    fn from(e: Error) -> Self {
        match e {
            Error::AlreadyExists(id) => Response::Error(format!("`{}` already exists", id)),
            Error::TooManyMetas(id, metas) => Response::TooManyMeta(id, metas),
            e => Response::Error(e.to_string()),
        }
    }
}
