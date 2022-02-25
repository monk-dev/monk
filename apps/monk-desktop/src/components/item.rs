use std::{cmp::Reverse, sync::Arc};

use dioxus::prelude::*;
use monk::types::{Item, MonkTrait, Search, SearchResult};
use ordered_float::NotNan;
use tracing::info;

use crate::{
    context::{use_monk_context, use_search_context},
    icons,
};

#[derive(PartialEq, Props)]
pub struct ItemProps {
    item: Item,
    #[props(default)]
    search_result: Option<SearchResult>,
}

pub fn Item(cx: Scope<ItemProps>) -> Element {
    // let (expanded, set_expanded) = use_state(&cx, || false);
    let item = &cx.props.item;

    let item_name = item.name.as_deref().unwrap_or_default();
    let item_created_at = item.created_at.format("%b %e, %Y").to_string();
    let item_summary = item.summary.as_deref();
    let item_comment = item.comment.as_deref();

    rsx!(cx,
        article {
            class: "p-1 border rounded-md bg-gradient-to-r from-gray-50 to-gray-100 hover:ring",
            div {
                class: "flex flex-col gap-1",
                // Header
                div {
                    class: "flex flex-row justify-between items-center",
                    h1 {
                        class: "font-serif font-bold text-lg",
                        "{item_name}"
                    },
                    div {
                        class: "flex flex-row gap-1 items-center",
                        small {
                            "Added on {item_created_at}"
                        }
                        // Open symbol
                        a {
                            href: "",
                            target: "_blank",
                            icons::Open {},
                        }
                    }
                }
                // Summary
                item_summary.map(|summary| {
                    rsx!(section {
                        class: "text-sm italic p-1 border rounded bg-white border-gray-300",
                        "{summary}"
                    })
                })
                div {
                    class: "flex flex-row justify-between",
                    item_comment.map(|comment| {
                        rsx!(section {
                            class: "text-sm",
                            "{comment}"
                        })
                    })
                }
                cx.props.search_result.as_ref().map(|res| {
                    rsx!(cx, pre {
                        "{res:?}"
                    })
                })
                // if *expanded {}
            }
        }
    )
}

pub fn SearchItems<'a>(cx: Scope<'a>) -> Element<'a> {
    let monk_ctx = use_monk_context(&cx);
    let search_ctx = use_search_context(&cx);

    let monk = Arc::clone(&monk_ctx.read().monk);
    let query = search_ctx.read().query().to_string();
    let fut_query = query.clone();
    let search_results = use_future(&cx, || async move {
        if !query.is_empty() {
            info!("searching in monk");
            let mut monk = monk.lock().await;
            monk.search(Search { query, count: None }).await.map(Some)
        } else {
            info!("skipping search");
            Ok(None)
        }
    });

    let (query, set_query) = use_state(&cx, || search_ctx.read().query().to_string());
    if query != &fut_query {
        info!("restarting future");

        set_query(fut_query);
        search_results.restart();
    }

    match search_results.value() {
        // Successful search, add contents
        Some(Ok(Some(results))) => {
            search_ctx.write().results = results
                .iter()
                .map(|res| (res.id.clone(), res.clone()))
                .collect();
        }
        // No search was preformed, render normally
        Some(Ok(None)) => {
            search_ctx.write().results.clear();
        }
        Some(Err(e)) => {
            return rsx!(cx, div { "Error Loading Items: {e}"});
        }
        // Loading
        None => {
            // is_loading = true;
        }
    };

    let items = &monk_ctx.read().items;
    let results = &search_ctx.read().results;
    let mut items: Vec<(_, _)> = items
        .iter()
        .filter_map(move |item| {
            if query.is_empty() || results.contains_key(&item.id) {
                Some((item, results.get(&item.id)))
            } else {
                None
            }
        })
        .collect();

    items.sort_by_key(|(item, search_result)| {
        let score = search_result
            .map(|res| NotNan::new(res.score).ok())
            .unwrap_or_default();

        Reverse((score, &item.created_at))
    });

    rsx!(
        cx,
        div {
            class: "flex flex-col gap-4 my-2 col-start-3 col-end-11 justify-center",
            items.into_iter().map(|(item, res)| rsx!(cx, Item { key: "{item.id}", item: item.clone(), search_result: res.cloned() }))
        }
    )
}
