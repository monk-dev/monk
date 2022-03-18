#![allow(non_snake_case)]
use std::sync::Arc;

use dioxus::prelude::*;
use monk::types::config::MonkConfig;
use monk::types::MonkTrait;
use monk::Monk;
use tokio::sync::Mutex;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

pub mod components;
pub mod context;
pub mod icons;

use crate::{
    components::{item::SearchItems, navbar::NavBar},
    context::{
        provide_monk_context, provide_search_context, use_monk_context, use_search_context,
        MonkContext,
    },
};

use self::components::item::Item;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(LevelFilter::INFO.into())
                .add_directive("sqlx::query=warn".parse()?)
                .add_directive("html5ever::serialize=off".parse()?)
                .add_directive("tantivy=warn".parse()?),
        )
        .finish()
        .init();

    let config = MonkConfig::default();
    let monk = monk_server::run(config).await?;

    dioxus::desktop::launch_with_props(App, AppProps { monk }, |c| {
        // c.with_window(|w| w.with_maximized(true))
        c
    });
    Ok(())
}

pub struct AppProps {
    monk: Arc<Mutex<Monk>>,
}

fn App(cx: Scope<AppProps>) -> Element {
    provide_monk_context(&cx, Arc::clone(&cx.props.monk));
    provide_search_context(&cx);

    let monk_ctx = use_monk_context(&cx);

    let monk_clone = Arc::clone(&monk_ctx.read().monk);
    let items = use_future(&cx, (), |()| async move {
        let mut monk = monk_clone.lock().await;
        monk.list(Default::default()).await.unwrap()
    });

    if let Some(items) = items.value() {
        monk_ctx.write().items = items
            .clone()
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect();
    }

    let tailwind = include_str!("../css/tailwindcss.js");

    let search_ctx = use_search_context(&cx);
    let query = search_ctx.read().query().to_string();

    rsx!(cx,
        script { "{tailwind}" }
        div {
            class: "grid gap-4 grid-cols grid-cols-12",
            NavBar {}
            // div {
            //     class: "flex flex-col gap-4 my-2 col-start-3 col-end-11 justify-center",
            //     // SearchItems {},
            // }
            // SearchItems { query: query }
        }
    )
}
