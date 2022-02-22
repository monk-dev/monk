use uuid::Uuid;

use crate::models::index::ExtractedInfo;
use crate::{Item, SearchResult, Tag};

pub trait Index {
    fn count(&self) -> anyhow::Result<usize>;

    fn search(&self, query: &str, count: usize) -> anyhow::Result<Vec<SearchResult>>;

    fn index(&mut self, item: &Item, tags: &[Tag]) -> anyhow::Result<()> {
        self.index_full(item, tags, Default::default())
    }

    fn index_full(&mut self, item: &Item, tags: &[Tag], extra: ExtractedInfo)
        -> anyhow::Result<()>;

    fn summarize(&mut self, text: &str) -> anyhow::Result<String>;

    fn remove(&mut self, id: Uuid) -> anyhow::Result<()>;
}
