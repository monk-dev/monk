use serde::{Deserialize, Serialize};

use crate::metadata::Meta;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Response {
    NewId(String),
    Item(Meta),
    List(Vec<Meta>),
    Error(String),
    NotFound(String),
    Unhandled,
    Ok,
}
