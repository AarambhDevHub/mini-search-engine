use crate::core::InvertedIndex;
use crate::storage::Storage;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct JsonStorage {
    file_path: String,
}

impl JsonStorage {
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }

    fn ensure_directory_exists(&self) -> Result<()> {
        if let Some(parent) = Path::new(&self.file_path).parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }
        Ok(())
    }
}

impl Storage for JsonStorage {
    fn save_index(&self, index: &InvertedIndex) -> Result<()> {
        self.ensure_directory_exists()?;

        let json =
            serde_json::to_string_pretty(index).context("Failed to serialize index to JSON")?;

        fs::write(&self.file_path, json)
            .with_context(|| format!("Failed to write index to file: {}", self.file_path))
    }

    fn load_index(&self) -> Result<Option<InvertedIndex>> {
        if !Path::new(&self.file_path).exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&self.file_path)
            .with_context(|| format!("Failed to read index from file: {}", self.file_path))?;

        let index = serde_json::from_str(&json).context("Failed to deserialize index from JSON")?;

        Ok(Some(index))
    }

    fn clear(&self) -> Result<()> {
        if Path::new(&self.file_path).exists() {
            fs::remove_file(&self.file_path)
                .with_context(|| format!("Failed to remove index file: {}", self.file_path))?;
        }
        Ok(())
    }
}
