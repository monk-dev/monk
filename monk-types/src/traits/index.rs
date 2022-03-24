use uuid::Uuid;

use crate::models::index::ExtractedInfo;
use crate::{Item, SearchResult};

pub trait Index {
    fn count(&self) -> anyhow::Result<usize>;

    fn search(&self, query: &str, count: usize) -> anyhow::Result<Vec<SearchResult>>;

    fn index(&mut self, item: &Item) -> anyhow::Result<()> {
        self.index_full(item, Default::default())
    }

    fn index_full(&mut self, item: &Item, extra: ExtractedInfo) -> anyhow::Result<()>;

    fn summarize(&mut self, text: &str) -> anyhow::Result<String>;

    fn remove(&mut self, id: Uuid) -> anyhow::Result<()>;
}
