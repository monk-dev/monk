use serde::{Deserialize, Serialize};
use url::Url;

use tokio::sync::oneshot::Sender;

use crate::error::Error;
use crate::metadata::Meta;

#[derive(Debug)]
pub struct CliRequest {
    request: RequestBody,
    response: Option<Sender<Result<ResponseBody, Error>>>,
}

impl CliRequest {
    pub fn new(
        request: RequestBody,
        response: Option<Sender<Result<ResponseBody, Error>>>,
    ) -> CliRequest {
        Self { request, response }
    }

    pub fn body(&self) -> &RequestBody {
        &self.request
    }

    pub fn response(&self) -> &Option<Sender<Result<ResponseBody, Error>>> {
        &self.response
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestBody {
    Add { name: String, url: Url },
    List { count: usize },
    Stop,
}

impl RequestBody {
    pub fn gives_response(&self) -> bool {
        match self {
            RequestBody::Add { .. } | RequestBody::Stop => false,
            RequestBody::List { .. } => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseBody {
    Entries(Vec<Meta>),
    Ok,
    Error(String),
}
