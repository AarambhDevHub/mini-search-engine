use crate::Document;
use crate::core::{InvertedIndex, TfIdfRanker};
use crate::crawler::{FileCrawler, WebCrawler};
use crate::storage::Storage;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SearchEngine {
    index: Arc<RwLock<InvertedIndex>>,
    storage: Arc<dyn Storage>,
}

impl SearchEngine {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            index: Arc::new(RwLock::new(InvertedIndex::new())),
            storage,
        }
    }

    pub async fn load_index(&self) -> Result<()> {
        let loaded_index = self
            .storage
            .load_index()
            .context("Failed to load index from storage")?;

        if let Some(index) = loaded_index {
            let mut current_index = self.index.write().await;
            *current_index = index;
            log::info!(
                "Loaded existing index with {} documents",
                current_index.total_documents
            );
        }

        Ok(())
    }

    pub async fn save_index(&self) -> Result<()> {
        let index = self.index.read().await;
        self.storage
            .save_index(&index)
            .context("Failed to save index to storage")
    }

    pub async fn index_directory(&self, directory_path: &str) -> Result<usize> {
        let crawler = FileCrawler::new(directory_path);
        let documents = crawler.crawl().context("Failed to crawl directory")?;

        let document_count = documents.len();
        let mut index = self.index.write().await;

        for document in documents {
            log::debug!("Indexing document: {}", document.title);
            index.add_document(document);
        }

        log::info!(
            "Indexed {} documents from {}",
            document_count,
            directory_path
        );
        Ok(document_count)
    }

    pub async fn search(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> Result<Vec<crate::core::ranking::SearchResult>> {
        let index = self.index.read().await;
        let results = TfIdfRanker::rank_documents(&index, query, limit);

        log::info!("Search for '{}' returned {} results", query, results.len());
        Ok(results)
    }

    pub async fn clear_index(&self) -> Result<()> {
        let mut index = self.index.write().await;
        *index = InvertedIndex::new();

        self.storage.clear().context("Failed to clear storage")?;

        log::info!("Index cleared");
        Ok(())
    }

    pub async fn get_stats(&self) -> (usize, usize) {
        let index = self.index.read().await;
        (index.total_documents, index.index.len())
    }

    pub async fn index_web_pages(
        &self,
        start_urls: Vec<String>,
        max_pages: Option<usize>,
    ) -> Result<usize> {
        let mut crawler = WebCrawler::new();

        if let Some(max) = max_pages {
            crawler = crawler.with_max_pages(max);
        }

        let documents = crawler
            .crawl(start_urls)
            .await
            .context("Failed to crawl web pages")?;

        let document_count = documents.len();
        let mut index = self.index.write().await;

        for document in documents {
            log::debug!("Indexing web document: {}", document.title);
            index.add_document(document);
        }

        log::info!("Indexed {} web documents", document_count);
        Ok(document_count)
    }

    pub async fn index_website(&self, base_url: String, max_pages: Option<usize>) -> Result<usize> {
        let url = url::Url::parse(&base_url).context("Invalid base URL")?;

        let domain = url.domain().context("No domain in URL")?.to_string();

        let crawler = WebCrawler::new()
            .with_max_pages(max_pages.unwrap_or(50))
            .with_allowed_domains(vec![domain])
            .with_delay(2000); // 2 second delay for single domain crawling

        let documents = crawler
            .crawl(vec![base_url])
            .await
            .context("Failed to crawl website")?;

        let document_count = documents.len();
        let mut index = self.index.write().await;

        for document in documents {
            log::debug!("Indexing web document: {}", document.title);
            index.add_document(document);
        }

        log::info!("Indexed {} documents from website", document_count);
        Ok(document_count)
    }

    pub async fn list_all_documents(&self) -> Vec<Document> {
        let index = self.index.read().await;
        index.get_all_documents().into_iter().cloned().collect()
    }
}
