use crate::search::SearchEngine;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "search-engine")]
#[command(about = "A mini search engine CLI")]
pub struct CliApp {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Index documents from a directory
    Index {
        /// Directory path to index
        #[arg(short, long)]
        directory: String,
    },
    /// Search for documents
    Search {
        /// Search query
        #[arg(short, long)]
        query: String,
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Clear the search index
    Clear,
    /// Show index statistics
    Stats,

    /// Index web pages from URLs
    IndexWeb {
        /// URLs to start crawling from
        #[arg(short, long, value_delimiter = ',')]
        urls: Vec<String>,
        /// Maximum number of pages to crawl
        #[arg(short, long, default_value = "50")]
        max_pages: usize,
    },

    /// Index an entire website (single domain)
    IndexSite {
        /// Base URL of the website
        #[arg(short, long)]
        url: String,
        /// Maximum number of pages to crawl
        #[arg(short, long, default_value = "100")]
        max_pages: usize,
    },

    /// List all indexed documents
    List,
}

impl CliApp {
    pub async fn run(self, engine: &SearchEngine) -> Result<()> {
        match self.command {
            Commands::Index { directory } => {
                println!("Indexing documents from: {}", directory);
                let count = engine.index_directory(&directory).await?;
                engine.save_index().await?;
                println!("Successfully indexed {} documents", count);
            }

            Commands::Search { query, limit } => {
                println!("Searching for: '{}'", query);
                let results = engine.search(&query, Some(limit)).await?;

                if results.is_empty() {
                    println!("No results found");
                } else {
                    println!("Found {} results:\n", results.len());

                    for (i, result) in results.iter().enumerate() {
                        println!(
                            "{}. {} (Score: {:.4})",
                            i + 1,
                            result.document.title,
                            result.score
                        );
                        println!("   Path: {}", result.document.path);
                        println!("   Snippet: {}", result.snippet);
                        println!();
                    }
                }
            }

            Commands::Clear => {
                engine.clear_index().await?;
                println!("Index cleared successfully");
            }

            Commands::Stats => {
                let (doc_count, term_count) = engine.get_stats().await;
                println!("Index Statistics:");
                println!("  Documents: {}", doc_count);
                println!("  Unique terms: {}", term_count);
            }

            Commands::IndexWeb { urls, max_pages } => {
                println!(
                    "Crawling web pages from {} URLs (max {} pages)",
                    urls.len(),
                    max_pages
                );
                let count = engine.index_web_pages(urls, Some(max_pages)).await?;
                engine.save_index().await?;
                println!("Successfully indexed {} web documents", count);
            }

            Commands::IndexSite { url, max_pages } => {
                println!("Crawling website: {} (max {} pages)", url, max_pages);
                let count = engine.index_website(url, Some(max_pages)).await?;
                engine.save_index().await?;
                println!("Successfully indexed {} documents from website", count);
            }

            Commands::List => {
                let documents = engine.list_all_documents().await;
                println!("Indexed Documents ({} total):\n", documents.len());

                for (i, doc) in documents.iter().enumerate() {
                    println!("{}. Title: {}", i + 1, doc.title);
                    println!("   URL: {}", doc.path);
                    println!("   Word count: {}", doc.word_count);
                    println!(
                        "   Content preview: {}...\n",
                        doc.content.chars().take(200).collect::<String>()
                    );
                }
            }
        }

        Ok(())
    }
}
