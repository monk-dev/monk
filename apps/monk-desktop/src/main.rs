#![allow(non_snake_case)]
use std::sync::Arc;

use dioxus::prelude::*;
use monk::types::config::MonkConfig;
use monk::types::MonkTrait;
use monk::Monk;
use tokio::sync::Mutex;

pub mod components;

use self::components::item::Item;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = MonkConfig::default();
    let monk = monk_server::run(config).await?;

    dioxus::desktop::launch_with_props(App, AppProps { monk }, |c| c);
    Ok(())
}

pub struct AppProps {
    monk: Arc<Mutex<Monk>>,
}

fn App(cx: Scope<AppProps>) -> Element {
    let monk_clone = Arc::clone(&cx.props.monk);
    let items = use_future(&cx, || async move {
        let mut monk = monk_clone.lock().await;
        monk.list(Default::default()).await.unwrap()
    });

    let items = items.value();
    let items = items
        .into_iter()
        .map(|items| items.into_iter().map(|item| rsx!(Item { item: item })))
        .flatten();

    rsx!(cx, ul { items })
}
