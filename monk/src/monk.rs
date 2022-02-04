use monk_dl::MonkDownloader;
use monk_index::{MonkExtractor, TantivyIndex};
use monk_sqlite::MonkSqlite;
use monk_types::config::MonkConfig;
use monk_types::{
    AddItem, Blob, CreateLink, DeleteItem, DeleteLink, Downloader, EditItem, Extractor, GetBlob,
    GetItem, Index, Item, LinkedItems, ListItem, MonkTrait, Search, SearchResult, Store,
};

pub struct Monk {
    pub config: MonkConfig,
    pub store: Box<dyn Store + Send + Sync + 'static>,
    pub index: Box<dyn Index + Send + Sync + 'static>,
    pub extractor: Box<dyn Extractor + Send + Sync + 'static>,
    pub downloader: Box<dyn Downloader + Send + Sync + 'static>,
}

impl Monk {
    pub async fn from_config(config: MonkConfig) -> anyhow::Result<Self> {
        let store = Box::new(MonkSqlite::from_config(&config.store).await?);
        let index = Box::new(TantivyIndex::from_config(&config.index).await?);
        let extractor = Box::new(MonkExtractor::default());
        let downloader = Box::new(MonkDownloader::from_config(&config.download).await?);

        Ok(Self {
            config,
            store,
            index,
            extractor,
            downloader,
        })
    }
}

#[async_trait::async_trait]
impl MonkTrait for Monk {
    async fn add(&mut self, add: AddItem) -> anyhow::Result<Item> {
        let item = self
            .store
            .add_item(add.name, add.url, add.comment, add.tags)
            .await?;

        let blob = if self.config.download.download_on_add {
            Some(match self.downloader.download(&*self.store, &item).await {
                Ok(blob) => blob,
                Err(error) => {
                    tracing::error!(%error, "unable to download item");
                    return Ok(item);
                }
            })
        } else {
            None
        };

        if self.config.index.index_on_add {
            let extracted = self.extractor.extract_info(&item, blob.as_ref()).await?;
            let tags = self.store.item_tags(item.id.clone()).await?;

            // remove any previous information
            self.index.remove(item.id.clone())?;

            self.index
                .index_full(&item, &tags, extracted.unwrap_or_default())?;
        }

        Ok(item)
    }

    async fn get(&mut self, get: GetItem) -> anyhow::Result<Option<Item>> {
        self.store.item(get.id.parse()?).await.map(Into::into)
    }

    async fn get_blob(&mut self, get: GetBlob) -> anyhow::Result<Option<Blob>> {
        match get {
            GetBlob::ItemId(item_id) => {
                self.store.item_blob(item_id.parse()?).await.map(Into::into)
            }
            GetBlob::BlobId(blob_id) => self.store.blob(blob_id.parse()?).await.map(Into::into),
        }
    }

    async fn list(&mut self, list: ListItem) -> anyhow::Result<Vec<Item>> {
        self.store.list_items().await.map(|items| {
            items
                .into_iter()
                .take(list.count.unwrap_or(usize::MAX))
                .collect()
        })
    }

    async fn edit(&mut self, edit: EditItem) -> anyhow::Result<Option<Item>> {
        self.store
            .update_item(edit.id.parse()?, edit.name, edit.url, edit.comment)
            .await
    }

    async fn delete(&mut self, delete: DeleteItem) -> anyhow::Result<Option<Item>> {
        self.store.delete_item(delete.id.parse()?).await
    }

    async fn linked_items(&mut self, item: LinkedItems) -> anyhow::Result<Vec<String>> {
        self.store.linked_items(item.id.parse()?).await
    }

    async fn link(&mut self, link: CreateLink) -> anyhow::Result<()> {
        self.store
            .create_link(link.a.parse()?, link.b.parse()?)
            .await
    }

    async fn unlink(&mut self, link: DeleteLink) -> anyhow::Result<()> {
        self.store
            .delete_link(link.a.parse()?, link.b.parse()?)
            .await
    }

    async fn search(&mut self, search: Search) -> anyhow::Result<Vec<SearchResult>> {
        self.index
            .search(&search.query, search.count.unwrap_or(usize::MAX))
    }
}
