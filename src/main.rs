use clap::Parser;
use mini_search_engine::{
    api::{CliApp, HttpServer},
    search::SearchEngine,
    storage::SledStorage,
};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Check if we should run in HTTP server mode
    if args.len() > 1 && args[1] == "server" {
        let port = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3030);

        // FIX: Use the SAME storage as CLI (Sled instead of JSON)
        let storage = Arc::new(SledStorage::new("data/index/search_index.db")?);
        let engine = Arc::new(SearchEngine::new(storage));

        // FIX: Load existing index if available
        println!("📚 Loading existing search index...");
        match engine.load_index().await {
            Ok(()) => {
                let (doc_count, term_count) = engine.get_stats().await;
                println!(
                    "✅ Loaded index: {} documents, {} terms",
                    doc_count, term_count
                );

                if doc_count == 0 {
                    println!(
                        "⚠️  Warning: No documents in index. Use CLI to index some content first:"
                    );
                    println!(
                        "   cargo run -- index-site --url \"https://rust-lang.org\" --max-pages 2"
                    );
                }
            }
            Err(e) => {
                println!("⚠️  Could not load existing index: {}", e);
                println!("   Starting with empty index. Use CLI or API to add content.");
            }
        }

        HttpServer::run(engine, port).await;
    } else {
        // CLI mode - keep existing logic
        let cli = CliApp::parse();

        // Use Sled storage for CLI mode
        let storage = Arc::new(SledStorage::new("data/index/search_index.db")?);
        let engine = SearchEngine::new(storage);

        // Load existing index if available
        engine.load_index().await?;

        cli.run(&engine).await?;
    }

    Ok(())
}
