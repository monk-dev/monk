use std::sync::Arc;

use dioxus::prelude::*;
use monk::{types::Item, Monk};
use tokio::sync::Mutex;

pub struct MonkContext {
    pub monk: Arc<Mutex<Monk>>,
    pub items: Vec<Item>,
}

pub fn provide_monk_context(cx: &ScopeState, monk: Arc<Mutex<Monk>>) {
    use_context_provider(cx, || MonkContext {
        monk,
        items: Vec::new(),
    });
}

pub fn use_monk_context(cx: &ScopeState) -> UseSharedState<MonkContext> {
    use_context(cx).unwrap()
}
