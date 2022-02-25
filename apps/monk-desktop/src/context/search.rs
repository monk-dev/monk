use std::collections::HashMap;

use dioxus::prelude::*;
use monk::types::SearchResult;
use uuid::Uuid;

#[derive(Default)]
pub struct SearchContext {
    pub previous_query: String,
    query: String,
    pub results: HashMap<Uuid, SearchResult>,
    pub loading: bool,
}

impl SearchContext {
    pub fn set_query(&mut self, query: String) {
        std::mem::swap(&mut self.previous_query, &mut self.query);
        self.query = query;
    }

    /// Get a reference to the search context's query.
    pub fn query(&self) -> &str {
        self.query.as_ref()
    }
}

pub fn provide_search_context(cx: &ScopeState) {
    use_context_provider(cx, || SearchContext::default());
}

pub fn use_search_context(cx: &ScopeState) -> UseSharedState<SearchContext> {
    use_context(cx).unwrap()
}
