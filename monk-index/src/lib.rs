mod extractor;
mod index;
pub mod schema;

use std::path::Path;

use monk_types::Index;

pub use self::extractor::*;
pub use self::index::*;

pub fn create_index(folder: impl AsRef<Path>) -> anyhow::Result<impl Index> {
    TantivyIndex::new(folder)
}
