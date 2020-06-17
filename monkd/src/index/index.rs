use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::{
    collector::TopDocs, directory::*, query::QueryParser, DocAddress, Document, Index as TIndex,
    IndexWriter, Opstamp, Term,
};

use crate::error::Error;
use crate::index::schema::*;
use crate::index::settings::IndexSettings;
use crate::metadata::Meta;

pub struct Index {
    index: TIndex,
    writer: IndexWriter,
}

impl Index {
    pub fn new(settings: &IndexSettings) -> Result<Self, Error> {
        let path = &settings.path;
        std::fs::create_dir_all(&path)?;

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

    pub fn search(&self, query: String) -> Result<Vec<String>, Error> {
        tracing::info!("[search] Query: {:?}", query);

        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        // tracing::info!("Got reader and searcher");

        let query_parser = QueryParser::for_index(&self.index, vec![ID, NAME, URL, COMMENT]);
        let query = query_parser.parse_query(&query)?;

        // tracing::info!("Parsed query");

        let resulting_docs: Vec<(f32, DocAddress)> =
            searcher.search(&query, &TopDocs::with_limit(3))?;

        // tracing::info!("Executed search");

        let docs: Result<Vec<_>, _> = resulting_docs
            .into_iter()
            .map(|(_score, address)| searcher.doc(address))
            .collect();

            
        let ids: Vec<_> = docs?
            .into_iter()
            .map(|doc| doc.get_first(ID).unwrap().text().unwrap().to_string()).collect();
            
        tracing::info!("Collected {} ids", ids.len());

        Ok(ids)
    }

    pub fn insert(&mut self, meta: &Meta) -> Result<Opstamp, Error> {
        tracing::info!("Indexing: {}", meta.id());
        
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

    pub fn delete(&mut self, id: impl AsRef<str>) -> Result<Opstamp, Error> {
        tracing::info!("[index] [delete] {}", id.as_ref());
        
        let term = Term::from_field_text(ID, id.as_ref());
        
        self.writer.delete_term(term);
        self.writer.commit().map_err(Into::into)
    }
}

impl Drop for Index {
    fn drop(&mut self) {
        let _ = self.writer.commit();
    }
}
