use crate::core::InvertedIndex;
use crate::storage::Storage;
use anyhow::{Context, Result};
use sled::{Db, Tree};
use std::path::Path;

pub struct SledStorage {
    #[allow(dead_code)]
    db: Db,
    tree: Tree,
}

impl SledStorage {
    pub fn new(db_path: &str) -> Result<Self> {
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let db = sled::open(db_path)
            .with_context(|| format!("Failed to open sled database at: {}", db_path))?;

        let tree = db
            .open_tree("inverted_index")
            .context("Failed to open inverted_index tree")?;

        Ok(Self { db, tree })
    }
}

impl Storage for SledStorage {
    fn save_index(&self, index: &InvertedIndex) -> Result<()> {
        let serialized = bincode::serialize(index).context("Failed to serialize index")?;

        self.tree
            .insert("index", serialized)
            .context("Failed to insert index into database")?;

        self.tree.flush().context("Failed to flush database")?;

        Ok(())
    }

    fn load_index(&self) -> Result<Option<InvertedIndex>> {
        match self
            .tree
            .get("index")
            .context("Failed to get index from database")?
        {
            Some(data) => {
                let index = bincode::deserialize(&data).context("Failed to deserialize index")?;
                Ok(Some(index))
            }
            None => Ok(None),
        }
    }

    fn clear(&self) -> Result<()> {
        self.tree.clear().context("Failed to clear database")?;

        self.tree
            .flush()
            .context("Failed to flush database after clear")?;

        Ok(())
    }
}
