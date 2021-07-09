use std::sync::Arc;

use rusqlite::Connection;
use tokio::sync::{Mutex, MutexGuard};

#[derive(Clone, Debug)]
pub struct DbConn {
    inner: Arc<Mutex<Connection>>,
}

impl DbConn {
    pub fn new(conn: Connection) -> Self {
        Self {
            inner: Arc::new(Mutex::new(conn)),
        }
    }

    pub async fn get<'a>(&'a self) -> MutexGuard<'a, Connection> {
        self.inner.lock().await
    }
}
