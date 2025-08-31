//! Mini Search Engine - Rust Implementation
//!
//! Developed by Aarambh Dev Hub
//!
//! A full-featured search engine library providing web crawling, document indexing,
//! TF-IDF ranking, and multiple search interfaces (CLI, HTTP API, Web UI).
//!
//! ## Features
//!
//! - Web and local file crawling
//! - Inverted index with TF-IDF scoring
//! - Persistent storage (Sled database, JSON)
//! - CLI and web interfaces
//! - Async/await support with Tokio
//!
//! ## Example
//!
//! ```
//! use mini_search_engine::{SearchEngine, SledStorage};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let storage = Arc::new(SledStorage::new("search_index.db")?);
//!     let engine = SearchEngine::new(storage);
//!
//!     // Index some content
//!     engine.index_directory("./documents").await?;
//!
//!     // Search
//!     let results = engine.search("rust programming", Some(10)).await?;
//!     println!("Found {} results", results.len());
//!
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod core;
pub mod crawler;
pub mod search;
pub mod storage;
pub mod web;

pub use core::{Document, InvertedIndex, TfIdfRanker, Tokenizer};
pub use crawler::FileCrawler;
pub use search::SearchEngine;
pub use storage::{JsonStorage, SledStorage, Storage};
