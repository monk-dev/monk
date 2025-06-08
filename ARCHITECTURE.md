# Monk Architecture Overview

**Monk** is a personal knowledge management system that has been refactored into a modular, trait-based architecture with clear separation of concerns. It provides capabilities for downloading, indexing, searching, and summarizing web content for offline access and analysis.

## Core Architecture Pattern

The system follows a **plugin-based architecture** using Rust traits, where the main `Monk` struct (`monk/src/monk.rs:11`) orchestrates different specialized components:

- **Store** (`monk-sqlite`): Data persistence layer
- **Index** (`monk-index`): Full-text search using Tantivy
- **Downloader** (`monk-dl`): Web content downloading
- **Extractor** (`monk-index`): Content extraction from downloaded files
- **Summarizer** (`monk-summary`): AI-powered text summarization

## Workspace Structure

The project is organized as a Cargo workspace with distinct responsibilities:

### Core Libraries

- **`monk`**: Main orchestration library that coordinates all components
- **`monk-types`**: Shared types, traits, and configuration definitions
- **`monk-sqlite`**: SQLite-based storage implementation using SeaORM
- **`monk-index`**: Tantivy-based search indexing and content extraction
- **`monk-dl`**: Web content downloading with Monolith integration
- **`monk-summary`**: PageRank-based text summarization engine

### Applications

- **`monk-cli`**: Command-line interface for user interactions
- **`monk-desktop`**: Dioxus-based desktop application (experimental)
- **`monk-browser-extension`**: Web browser extension for content capture

## Data Flow

The system processes content through a well-defined pipeline:

### 1. Add Item Workflow
```
CLI/UI → Monk::add() → Store → (optional) Download → Extract → Index → Summarize
```

When adding a new item:
1. User provides metadata (name, URL, tags, comment)
2. Item is stored in the database
3. If configured, content is automatically downloaded
4. Downloaded content is processed for text extraction
5. Extracted content is indexed for search
6. Content summary is generated using PageRank algorithm

### 2. Search Workflow
```
CLI/UI → Monk::search() → Index → Results with snippets
```

Search operations:
1. Query is processed by the search index
2. Results are ranked by relevance
3. Snippets are extracted with highlighting
4. Results include metadata and blob associations

### 3. Retrieve Workflow
```
CLI/UI → Monk::get() → Store → Item with blob/tags
```

Item retrieval:
1. Item is fetched from storage with full metadata
2. Associated tags and blob information are included
3. Links to related items are available

## Key Features

### Automatic Processing Pipeline
- **Download on Add**: Configurable automatic content downloading
- **Index on Add**: Automatic full-text indexing of extracted content
- **Summarize on Add**: Intelligent content summarization

### Linking System
- **Bidirectional Links**: Items can be linked together for relationship mapping
- **Link Traversal**: Navigate between related knowledge pieces
- **Link Management**: Create and delete relationships programmatically

### Full-Text Search
- **Multi-field Search**: Search across names, content, comments, and tags
- **Relevance Ranking**: Results ranked by search relevance
- **Snippet Highlighting**: Search terms highlighted in result snippets
- **Powered by Tantivy**: High-performance search index

### Content Processing
- **Multi-format Support**: HTML, PDF, and other document formats
- **Text Extraction**: Convert formatted content to searchable text
- **Metadata Extraction**: Preserve important document metadata
- **Content Sanitization**: Clean and normalize extracted text

### Smart Summarization
- **TF-IDF Analysis**: Term frequency-inverse document frequency scoring
- **PageRank Algorithm**: Graph-based sentence ranking for summary selection
- **Configurable Length**: Adjustable summary length and quality
- **Language Processing**: English language support with stemming and stop-word filtering

## Configuration

The system is highly configurable through `MonkConfig`:

```rust
pub struct MonkConfig {
    pub data_dir: PathBuf,           // Base directory for all data
    pub store: StoreConfig,          // Database configuration
    pub index: IndexConfig,          // Search index settings
    pub download: DownloadConfig,    // Download behavior settings
}
```

### Key Configuration Options
- **`download_on_add`**: Automatically download content when adding items
- **`index_on_add`**: Automatically index content for search
- **`summarize_on_add`**: Generate summaries during content processing

## Component Interactions

The architecture is designed for modularity and extensibility:

1. **Trait-based Design**: All major components implement well-defined traits
2. **Dependency Injection**: Components are injected into the main `Monk` struct
3. **Async Operations**: All I/O operations are asynchronous for performance
4. **Error Handling**: Comprehensive error handling with `anyhow::Result`
5. **Type Safety**: Strong typing throughout with UUID-based identifiers

## Storage Model

### Core Entities
- **Items**: Primary knowledge pieces with metadata
- **Tags**: Categorization and organization
- **Links**: Bidirectional relationships between items
- **Blobs**: Downloaded content files with metadata

### Database Schema
- **SQLite Backend**: Single-file database for portability
- **SeaORM Integration**: Type-safe database operations
- **Migration Support**: Database schema versioning

## Search Architecture

### Indexing Strategy
- **Document-based**: Each item becomes a searchable document
- **Multi-field**: Separate fields for names, content, comments
- **Real-time Updates**: Index updates on item modifications
- **Persistence**: Index stored on disk for fast startup

### Query Processing
- **Query Parsing**: Tantivy-based query processing
- **Field Weighting**: Different weights for different content types
- **Snippet Generation**: Contextual result snippets
- **Highlighting**: Search term highlighting in results

## Extension Points

The modular architecture supports easy extension:

1. **New Storage Backends**: Implement the `Store` trait
2. **Additional Downloaders**: Implement the `Downloader` trait
3. **Custom Extractors**: Implement the `Extractor` trait
4. **Alternative Search**: Implement the `Index` trait
5. **New Applications**: Use `MonkTrait` for different interfaces

This architecture ensures that Monk can evolve and adapt to new requirements while maintaining clean separation of concerns and high code quality.