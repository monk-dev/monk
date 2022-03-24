use std::path::Path;

use anyhow::Context;
use monk_types::{config::IndexConfig, Snippets};
use monk_types::{ExtractedInfo, Index, Item, SearchResult, Snippet};
use tantivy::query::QueryParser;
use tantivy::{collector::TopDocs, Document};
use tantivy::{directory::MmapDirectory, query::Query, Searcher};
use tantivy::{DocAddress, SnippetGenerator, Term};
use tracing::info;
use uuid::Uuid;

use crate::schema::{
    current_schema, BODY, COMMENT, EXTRA, FOUND, ID, NAME, SCHEMA_VERSION, TAG, TITLE, URL,
};

pub struct MonkIndex {
    index: tantivy::Index,
    writer: tantivy::IndexWriter,
}

impl MonkIndex {
    pub async fn from_config(
        data_dir: impl AsRef<Path>,
        config: &IndexConfig,
    ) -> anyhow::Result<Self> {
        MonkIndex::new(data_dir.as_ref().join(&config.path))
    }

    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        std::fs::create_dir_all(&path)?;
        tracing::info!("schema version: {}", SCHEMA_VERSION);

        let schema = current_schema();
        let mmap_dir = MmapDirectory::open(path).context("creating mmap dir")?;
        let index = tantivy::Index::open_or_create(mmap_dir, schema)
            .context("opening or creating index")?;

        let writer = index.writer(50_000_000).context("error making writer")?;

        Ok(MonkIndex { index, writer })
    }
}

impl Index for MonkIndex {
    fn count(&self) -> anyhow::Result<usize> {
        use tantivy::collector::Count;

        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let query = tantivy::query::AllQuery;
        let count = searcher.search(&query, &Count)?;

        Ok(count)
    }

    fn search(&self, query: &str, count: usize) -> anyhow::Result<Vec<SearchResult>> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(
            &self.index,
            vec![ID, NAME, URL, COMMENT, BODY, TITLE, EXTRA],
        );

        let query = query_parser.parse_query(&query)?;

        let resulting_docs: Vec<(f32, DocAddress)> =
            searcher.search(&query, &TopDocs::with_limit(count))?;

        let docs: Result<Vec<_>, _> = resulting_docs
            .into_iter()
            .map(|(score, address)| searcher.doc(address).map(|doc| (score, doc)))
            .collect();

        let docs = &docs?;

        let results = create_search_results(&searcher, &query, &docs)?;
        Ok(results)
    }

    fn index_full(&mut self, item: &Item, extra: ExtractedInfo) -> anyhow::Result<()> {
        tracing::info!("Indexing: {}", &item.id);

        let mut doc = tantivy::Document::new();

        doc.add_text(ID, &item.id);
        doc.add_text(NAME, &item.name);

        if let Some(url) = &item.url {
            doc.add_text(URL, &url);
        }

        if let Some(comment) = &item.comment {
            doc.add_text(COMMENT, comment);
        }

        doc.add_date(FOUND, &item.created_at);

        for tag in &item.tags {
            if !tag.tag.starts_with("/") {
                doc.add_facet(TAG, &format!("/{}", tag.tag));
            } else {
                doc.add_facet(TAG, &tag.tag);
            }
        }

        if let Some(title) = extra.title {
            doc.add_text(TITLE, title);
        } else {
            doc.add_text(TITLE, &item.name);
        }

        if let Some(body) = extra.body {
            doc.add_text(BODY, body);
        }

        if let Some(extra) = extra.extra {
            doc.add_text(EXTRA, extra);
        }

        self.writer.add_document(doc);
        self.writer.commit().context("committing item document")?;

        Ok(())
    }

    fn summarize(&mut self, _text: &str) -> anyhow::Result<String> {
        info!("summarizing text");
        Ok(String::new())
    }

    fn remove(&mut self, id: Uuid) -> anyhow::Result<()> {
        let term = Term::from_field_text(ID, &id.to_string());

        self.writer.delete_term(term);
        self.writer.commit().context("removing an item")?;

        Ok(())
    }
}

fn create_search_results(
    searcher: &Searcher,
    query: &dyn Query,
    docs: &[(f32, Document)],
) -> anyhow::Result<Vec<SearchResult>> {
    let mut name_generator = SnippetGenerator::create(&searcher, &*query, NAME)?;
    name_generator.set_max_num_chars(120);

    let mut body_generator = SnippetGenerator::create(&searcher, &*query, BODY)?;
    body_generator.set_max_num_chars(120);

    let mut comment_generator = SnippetGenerator::create(&searcher, &*query, COMMENT)?;
    comment_generator.set_max_num_chars(120);

    let results: Vec<SearchResult> = docs
        .iter()
        .flat_map(|(score, doc)| {
            let id = doc.get_first(ID)?.text()?.parse().ok()?;

            Some(SearchResult {
                id,
                score: *score,
                snippets: Snippets {
                    name: convert_snippet(name_generator.snippet_from_doc(&doc)),
                    body: convert_snippet(body_generator.snippet_from_doc(&doc)),
                    comment: convert_snippet(comment_generator.snippet_from_doc(&doc)),
                },
            })
        })
        .collect();

    Ok(results)
}

pub fn convert_snippet(snippet: tantivy::Snippet) -> Snippet {
    Snippet {
        fragment: snippet.fragments().to_string(),
        highlighted: snippet
            .highlighted()
            .iter()
            .map(|range| (range.start, range.end))
            .collect(),
    }
}

// pub struct Index {
//     index: TIndex,
//     folder: PathBuf,
//     writer: IndexWriter,
// }
