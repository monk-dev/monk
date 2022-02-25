use dioxus::{events::onkeyup, prelude::*};
use tracing::info;

use crate::{context::use_search_context, icons};

pub fn SearchBar<'i>(cx: Scope<'i>) -> Element {
    let search_ctx = use_search_context(&cx);

    let (query, set_query) = use_state(&cx, || search_ctx.read().query().to_string());
    if query != search_ctx.read().query() {
        search_ctx.write().set_query(query.to_string());
    }

    rsx!(cx, div {
        class: "flex flex-row items-center border rounded-full",
        div {
            class: "m-2",
            icons::Search {}
        }
        input {
            class: "grow focus:outline-none",
            placeholder: "Search",
            onchange: move |e| {
                info!("setting search_bar query");
                set_query(e.value.clone());
            }
        },
        (!search_ctx.read().query().is_empty()).then(|| {
            rsx!(cx, div {
                class: "p-2 hover:cursor-pointer",
                onclick: move |_| {
                    info!("clearing search_bar query");
                    set_query(String::new());
                },
                icons::X {}
            })
        })
    })
}
