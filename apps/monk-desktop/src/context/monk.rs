use std::{collections::HashMap, sync::Arc};

use dioxus::prelude::*;
use monk::{types::Item, Monk};
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct MonkContext {
    pub monk: Arc<Mutex<Monk>>,
    pub items: HashMap<Uuid, Item>,
}

pub fn provide_monk_context(cx: &ScopeState, monk: Arc<Mutex<Monk>>) {
    use_context_provider(cx, || MonkContext {
        monk,
        items: HashMap::new(),
    });
}

pub fn use_monk_context(cx: &ScopeState) -> UseSharedState<MonkContext> {
    use_context(cx).unwrap()
}

pub fn use_monk(cx: &ScopeState) -> Arc<Mutex<Monk>> {
    Arc::clone(&use_monk_context(cx).read().monk)
}
