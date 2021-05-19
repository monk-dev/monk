#![allow(unused_variables)]

pub mod http;
pub mod youtube;

use crate::error::Error;
use crate::index::Index;
use crate::metadata::{offline_store::OfflineData, Meta};
use crate::Response;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdapterType {
    Http,
    Youtube,
}

impl Default for AdapterType {
    fn default() -> Self {
        AdapterType::Http
    }
}

#[async_trait]
pub trait Adapter: Send {
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

    // Give a meta and an offline data, can an adapter handle
    // modifying the off line store for that meta data
    fn can_modify(&self, meta: &Meta, offline: Option<&OfflineData>) -> bool {
        false
    }

    /*
    / This indicates how willing an adapter is to handle some type
    / meta. The higher the score, the more capable the adapter is
    / at handling the incoming Meta. For example, a meta might have
    / a url of "https://www.youtube.com/watch?v=dQw4w9WgXcQ". The http
    / adapter would return "1" and the youtube adapter would return "5"
    / because it can handle youtube links better.
    */
    fn score_meta(&self, meta: &Meta) -> usize {
        0
    }

    fn adt_type(&self) -> AdapterType {
        AdapterType::Http
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
