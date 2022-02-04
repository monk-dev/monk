use crate::{Blob, ExtractedInfo, Item};

#[async_trait::async_trait]
pub trait Extractor {
    async fn extract_info(
        &self,
        item: &Item,
        blob: Option<&Blob>,
    ) -> anyhow::Result<Option<ExtractedInfo>>;
}
