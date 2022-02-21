use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use monk::types::config::MonkConfig;
use monk::types::MonkTrait;
use monk::Monk;
use tracing::info;

use crate::message::{Request, Response, ResponseMsg};

pub struct MonkServer {
    monk: Monk,
    rx: Receiver<Request>,
    tx: Sender<Response>,
}

impl MonkServer {
    pub async fn from_config(
        config: MonkConfig,
        tx: Sender<Response>,
        rx: Receiver<Request>,
    ) -> anyhow::Result<Self> {
        let monk = Monk::from_config(config).await?;

        Ok(Self { monk, rx, tx })
    }

    pub async fn run(mut self, cancel: Arc<AtomicBool>) {
        let five_ms = std::time::Duration::from_millis(5);

        loop {
            let msg = if let Ok(msg) = self.rx.try_recv() {
                msg
            } else {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }

                tokio::time::sleep(five_ms).await;
                continue;
            };

            info!(?msg, "server received");

            let resp = match msg {
                crate::message::RequestMsg::AddItem(a) => {
                    self.monk.add(a).await.map(ResponseMsg::Add)
                }
                crate::message::RequestMsg::GetItem(g) => {
                    self.monk.get(g).await.map(ResponseMsg::Get)
                }
                crate::message::RequestMsg::GetBlob(g) => {
                    self.monk.get_blob(g).await.map(ResponseMsg::GetBlob)
                }
                crate::message::RequestMsg::ListItem(l) => {
                    self.monk.list(l).await.map(ResponseMsg::List)
                }
                crate::message::RequestMsg::EditItem(e) => {
                    self.monk.edit(e).await.map(ResponseMsg::Edit)
                }
                crate::message::RequestMsg::DeleteItem(d) => {
                    self.monk.delete(d).await.map(ResponseMsg::Delete)
                }
                crate::message::RequestMsg::Search(s) => {
                    self.monk.search(s).await.map(ResponseMsg::Search)
                }
            };

            info!(?resp, "responding with");

            let _ = self.tx.send(resp);
        }
    }
}
