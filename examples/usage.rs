use mini_search_engine::{search::SearchEngine, storage::JsonStorage};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the search engine with JSON storage
    let storage = Arc::new(JsonStorage::new("examples/search_index.json".to_string()));
    let engine = SearchEngine::new(storage);

    // Index the sample documents
    println!("Indexing sample documents...");
    let count = engine.index_directory("examples/sample_docs").await?;
    println!("Indexed {} documents", count);

    // Save the index
    engine.save_index().await?;
    println!("Index saved");

    // Perform some searches
    println!("\n=== Search Results ===");

    let queries = vec!["rust programming", "machine learning", "web development"];

    for query in queries {
        println!("\nSearching for: '{}'", query);
        let results = engine.search(query, Some(3)).await?;

        for (i, result) in results.iter().enumerate() {
            println!(
                "  {}. {} (Score: {:.4})",
                i + 1,
                result.document.title,
                result.score
            );
            println!("     {}", result.snippet);
        }
    }

    // Show index statistics
    let (doc_count, term_count) = engine.get_stats().await;
    println!("\n=== Index Statistics ===");
    println!("Documents: {}", doc_count);
    println!("Unique terms: {}", term_count);

    Ok(())
}
