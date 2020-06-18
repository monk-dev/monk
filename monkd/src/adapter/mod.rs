pub mod http;

use crate::error::Error;
use crate::index::Index;
use crate::metadata::{offline_store::OfflineData, Meta};
use crate::{Request, Response};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdapterSlug {
    Http,
}

#[async_trait]
pub trait Adapter
where
    Self: Send,
{
    // /// Initialize an adapter with the provided configuration string. If `None`
    // /// is returned, the `Self::default()` implementation will be used. The provided
    // /// `Sink<Request>` can be used to send more messages to the event loop. This can
    // /// be used, for example, to create more metadata elements when `handle_add` or
    // /// `init_download` is called.
    // async fn initialize(&mut self, config: Option<String>, sender: Sender<Request>) -> Option<Result<Self, Error>> {
    //     None
    // }

    async fn handle_add(&mut self, meta: &Meta) -> Option<Result<Response, Error>> {
        None
    }

    async fn init_download(
        &mut self,
        meta: Option<&Meta>,
        offline: Option<OfflineData>,
    ) -> Option<OfflineData> {
        None
    }

    async fn handle_download<'s, 'a>(
        &'s mut self,
        meta: Option<&'a Meta>,
        offline: Option<OfflineData>,
    ) -> Option<Result<Response, Error>> {
        None
    }

    async fn handle_delete(&mut self, meta: &Meta) -> Option<Result<(), Error>> {
        None
    }

    async fn handle_index(
        &mut self,
        meta: &Meta,
        offline: Option<&OfflineData>,
        index: &mut Index,
    ) -> Option<Result<(), Error>> {
        None
    }

    async fn shutdown(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
