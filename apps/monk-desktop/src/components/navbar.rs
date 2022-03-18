use dioxus::prelude::*;
use tracing::info;

use crate::components::search_bar::SearchBar;

pub fn NavBar<'i>(cx: Scope<'i>) -> Element {
    info!("drawing navbar");

    rsx!(cx, div {
        class: "col-span-full border p-1",
        div {
            class: "grid grid-cols-3 gap-2 items-center",
            div {
                class: "text-2xl font-semibold text-red-600 tracking-widest",
                "MONK"
            }
            SearchBar {}
            div {
                class: "justify-self-end",
                "Div B"
            }
        }
    })
}
