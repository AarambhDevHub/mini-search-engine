use crate::core::Document;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct FileCrawler {
    root_path: PathBuf,
}

impl FileCrawler {
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
        }
    }

    pub fn crawl(&self) -> Result<Vec<Document>> {
        let mut documents = Vec::new();

        for entry in WalkDir::new(&self.root_path) {
            let entry = entry.context("Failed to read directory entry")?;

            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "txt" || extension == "md" {
                        match self.process_file(entry.path()) {
                            Ok(doc) => documents.push(doc),
                            Err(e) => {
                                log::warn!("Failed to process file {:?}: {}", entry.path(), e);
                            }
                        }
                    }
                }
            }
        }

        Ok(documents)
    }

    fn process_file(&self, path: &Path) -> Result<Document> {
        let content =
            fs::read_to_string(path).with_context(|| format!("Failed to read file: {:?}", path))?;

        let title = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("Untitled")
            .to_string();

        let path_str = path.to_string_lossy().to_string();

        Ok(Document::new(title, content, path_str))
    }
}
