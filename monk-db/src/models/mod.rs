use std::sync::Arc;

use rusqlite::Connection;
use tokio::sync::{Mutex, MutexGuard};

pub mod article;
pub mod article_tag;
pub mod tag;

#[derive(Clone)]
pub struct DbConn {
    inner: Arc<Mutex<Connection>>,
}

impl DbConn {
    pub async fn get<'a>(&'a self) -> MutexGuard<'a, Connection> {
        self.inner.lock().await
    }
}
