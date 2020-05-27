use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::{directory::*, Document, Index as TIndex, IndexWriter, Opstamp};

use crate::error::Error;
use crate::index::schema::{current_schema, SCHEMA_VERSION};
use crate::index::settings::IndexSettings;
use crate::metadata::Meta;

pub struct Index {
    index: TIndex,
    writer: IndexWriter,
}

impl Index {
    pub fn new(settings: &IndexSettings) -> Result<Self, Error> {
        let path = &settings.path;

        tracing::info!("Schema Version: {}", SCHEMA_VERSION);
        let schema = current_schema();
        let mmap_dir = MmapDirectory::open(path).map_err(|e| Error::Tantivy(e.to_string()))?;
        let index =
            TIndex::open_or_create(mmap_dir, schema).map_err(|e| Error::Tantivy(e.to_string()))?;

        let writer = index
            .writer(50_000_000)
            .map_err(|e| Error::Tantivy(e.to_string()))?;

        Ok(Index { index, writer })
    }

    pub fn insert(&mut self, meta: &Meta) -> Result<Opstamp, Error> {
        let mut doc = Document::new();
        let schema = current_schema();

        doc.add_text(schema.get_field("id").unwrap(), meta.id());

        if let Some(name) = meta.name() {
            doc.add_text(schema.get_field("name").unwrap(), name);
        }

        if let Some(url) = meta.url() {
            doc.add_text(schema.get_field("url").unwrap(), &url.to_string());
        }

        if let Some(comment) = meta.comment() {
            doc.add_text(schema.get_field("comment").unwrap(), comment);
        }

        let found = meta.found();
        doc.add_date(schema.get_field("found").unwrap(), found);

        self.writer.add_document(doc);
        self.writer
            .commit()
            .map_err(|e| Error::Tantivy(e.to_string()))
    }
}

impl Drop for Index {
    fn drop(&mut self) {
        let _ = self.writer.commit();
    }
}
