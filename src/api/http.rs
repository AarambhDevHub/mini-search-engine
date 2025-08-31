use crate::search::SearchEngine;
use crate::web::SEARCH_PAGE_HTML;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::Filter;

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct SearchResponse {
    query: String,
    results: Vec<SearchResultJson>,
    total: usize,
}

#[derive(Serialize)]
struct SearchResultJson {
    title: String,
    path: String,
    score: f64,
    snippet: String,
}

#[derive(Serialize)]
struct StatsResponse {
    documents: usize,
    terms: usize,
}

pub struct HttpServer;

impl HttpServer {
    pub async fn run(engine: Arc<SearchEngine>, port: u16) {
        let search_engine = Arc::clone(&engine);

        // Serve the main search page at root
        let homepage = warp::path::end().map(|| warp::reply::html(SEARCH_PAGE_HTML));

        // API routes
        let search = warp::path("search")
            .and(warp::get())
            .and(warp::query::<SearchQuery>())
            .and(with_engine(Arc::clone(&search_engine)))
            .and_then(handle_search);

        let status = warp::path("status")
            .and(warp::get())
            .and(with_engine(Arc::clone(&search_engine)))
            .and_then(handle_status);

        let stats = warp::path("stats")
            .and(warp::get())
            .and(with_engine(Arc::clone(&search_engine)))
            .and_then(handle_stats);

        let index = warp::path("index")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_engine(Arc::clone(&search_engine)))
            .and_then(handle_index);

        let index_web = warp::path("index-web")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_engine(Arc::clone(&search_engine)))
            .and_then(handle_index_web);

        let index_site = warp::path("index-site")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_engine(Arc::clone(&search_engine)))
            .and_then(handle_index_site);

        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST"]);

        // Combine all routes - homepage first, then API routes
        let routes = homepage
            .or(status)
            .or(search)
            .or(stats)
            .or(index)
            .or(index_web)
            .or(index_site)
            .with(cors)
            .with(warp::log("search_engine"));

        println!("🚀 Mini Search Engine Server Starting");
        println!("📍 Address: http://localhost:{}", port);
        println!("🌐 Web Interface: http://localhost:{}/", port);
        println!("🔧 API Endpoints:");
        println!("   GET  /search?q=<query>&limit=<limit>");
        println!("   GET  /stats");
        println!("   POST /index {{\"directory\": \"/path/to/docs\"}}");
        println!("   POST /index-web {{\"urls\": [\"url1\", \"url2\"], \"max_pages\": 50}}");
        println!("   POST /index-site {{\"url\": \"https://example.com\", \"max_pages\": 100}}");
        println!();

        warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    }
}

fn with_engine(
    engine: Arc<SearchEngine>,
) -> impl Filter<Extract = (Arc<SearchEngine>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || Arc::clone(&engine))
}

async fn handle_status(engine: Arc<SearchEngine>) -> Result<impl warp::Reply, warp::Rejection> {
    let (documents, terms) = engine.get_stats().await;

    Ok(warp::reply::json(&serde_json::json!({
        "status": "healthy",
        "index": {
            "documents": documents,
            "terms": terms,
            "ready": documents > 0
        },
        "message": if documents == 0 {
            "No documents indexed. Use CLI or API to add content."
        } else {
            "Search index ready"
        }
    })))
}

async fn handle_search(
    query: SearchQuery,
    engine: Arc<SearchEngine>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Debug logging
    let (doc_count, term_count) = engine.get_stats().await;
    println!(
        "🔍 Search request: '{}' | Index: {} docs, {} terms",
        query.q, doc_count, term_count
    );

    // Check if index is empty
    if doc_count == 0 {
        println!("❌ No documents in index!");
        return Ok(warp::reply::json(&SearchResponse {
            query: query.q,
            results: vec![],
            total: 0,
        }));
    }

    let results = engine.search(&query.q, query.limit).await.map_err(|e| {
        println!("❌ Search error: {:?}", e);
        warp::reject::custom(SearchError)
    })?;

    println!("📊 Found {} results for '{}'", results.len(), query.q);

    let json_results: Vec<SearchResultJson> = results
        .into_iter()
        .map(|r| {
            println!("  - {} (score: {:.4})", r.document.title, r.score);
            SearchResultJson {
                title: r.document.title,
                path: r.document.path,
                score: r.score,
                snippet: r.snippet,
            }
        })
        .collect();

    let response = SearchResponse {
        query: query.q,
        total: json_results.len(),
        results: json_results,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_stats(engine: Arc<SearchEngine>) -> Result<impl warp::Reply, warp::Rejection> {
    let (documents, terms) = engine.get_stats().await;

    let response = StatsResponse { documents, terms };
    Ok(warp::reply::json(&response))
}

#[derive(Deserialize)]
struct IndexRequest {
    directory: String,
}

#[derive(Deserialize)]
struct IndexWebRequest {
    urls: Vec<String>,
    max_pages: Option<usize>,
}

#[derive(Deserialize)]
struct IndexSiteRequest {
    url: String,
    max_pages: Option<usize>,
}

async fn handle_index(
    request: IndexRequest,
    engine: Arc<SearchEngine>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let count = engine
        .index_directory(&request.directory)
        .await
        .map_err(|_| warp::reject::custom(SearchError))?;

    engine
        .save_index()
        .await
        .map_err(|_| warp::reject::custom(SearchError))?;

    Ok(warp::reply::json(&serde_json::json!({
        "indexed_documents": count,
        "message": "Documents indexed successfully"
    })))
}

async fn handle_index_web(
    request: IndexWebRequest,
    engine: Arc<SearchEngine>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let count = engine
        .index_web_pages(request.urls, request.max_pages)
        .await
        .map_err(|_| warp::reject::custom(SearchError))?;

    engine
        .save_index()
        .await
        .map_err(|_| warp::reject::custom(SearchError))?;

    Ok(warp::reply::json(&serde_json::json!({
        "indexed_documents": count,
        "message": "Web pages indexed successfully"
    })))
}

async fn handle_index_site(
    request: IndexSiteRequest,
    engine: Arc<SearchEngine>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let count = engine
        .index_website(request.url, request.max_pages)
        .await
        .map_err(|_| warp::reject::custom(SearchError))?;

    engine
        .save_index()
        .await
        .map_err(|_| warp::reject::custom(SearchError))?;

    Ok(warp::reply::json(&serde_json::json!({
        "indexed_documents": count,
        "message": "Website indexed successfully"
    })))
}

#[derive(Debug)]
struct SearchError;

impl warp::reject::Reject for SearchError {}
