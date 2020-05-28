use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::metadata::offline_store::Status;
use crate::metadata::Meta;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Response {
    NewId(String),
    Item(Meta),
    List(Vec<Meta>),
    Error(String),
    NotFound(String),
    Status(String, Status),
    OpenStatus(String, Status),
    Open(PathBuf),
    Unhandled,
    Ok,
}
