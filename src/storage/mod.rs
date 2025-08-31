pub mod json_storage;
pub mod sled_storage;

use crate::core::InvertedIndex;
use anyhow::Result;

pub trait Storage: Send + Sync {
    fn save_index(&self, index: &InvertedIndex) -> Result<()>;
    fn load_index(&self) -> Result<Option<InvertedIndex>>;
    fn clear(&self) -> Result<()>;
}

pub use json_storage::JsonStorage;
pub use sled_storage::SledStorage;
