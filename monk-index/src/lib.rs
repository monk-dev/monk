mod extractor;
mod index;
pub mod schema;

use monk_types::Index;
use std::path::Path;

pub use self::extractor::*;
pub use self::index::*;

pub fn create_index(folder: impl AsRef<Path>) -> anyhow::Result<impl Index> {
    MonkIndex::new(folder)
}
