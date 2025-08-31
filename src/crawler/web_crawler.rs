use crate::core::Document;
use anyhow::{Context, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

pub struct WebCrawler {
    client: Client,
    max_pages: usize,
    delay_ms: u64,
    allowed_domains: HashSet<String>,
}

impl WebCrawler {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("MiniSearchEngine/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            max_pages: 100,
            delay_ms: 1000, // 1 second delay between requests
            allowed_domains: HashSet::new(),
        }
    }

    pub fn with_max_pages(mut self, max_pages: usize) -> Self {
        self.max_pages = max_pages;
        self
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    pub fn with_allowed_domains(mut self, domains: Vec<String>) -> Self {
        self.allowed_domains = domains.into_iter().collect();
        self
    }

    pub async fn crawl(&self, start_urls: Vec<String>) -> Result<Vec<Document>> {
        let mut documents = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Add start URLs to queue
        for url in start_urls {
            if let Ok(parsed_url) = Url::parse(&url) {
                queue.push_back(parsed_url);
            }
        }

        let mut pages_crawled = 0;

        while let Some(url) = queue.pop_front() {
            if pages_crawled >= self.max_pages {
                break;
            }

            let url_string = url.to_string();
            if visited.contains(&url_string) {
                continue;
            }

            if !self.is_allowed_domain(&url) {
                continue;
            }

            visited.insert(url_string.clone());

            match self.fetch_and_parse(&url).await {
                Ok((document, links)) => {
                    log::info!("Crawled: {}", document.title);
                    documents.push(document);
                    pages_crawled += 1;

                    // Add found links to queue
                    for link in links {
                        if !visited.contains(&link.to_string()) {
                            queue.push_back(link);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to crawl {}: {}", url, e);
                }
            }

            // Be polite - add delay between requests
            sleep(Duration::from_millis(self.delay_ms)).await;
        }

        log::info!("Web crawling completed. Crawled {} pages", pages_crawled);
        Ok(documents)
    }

    async fn fetch_and_parse(&self, url: &Url) -> Result<(Document, Vec<Url>)> {
        // Fetch the page
        let response = self
            .client
            .get(url.as_str())
            .send()
            .await
            .context("Failed to fetch URL")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let html = response
            .text()
            .await
            .context("Failed to read response body")?;

        // Parse HTML
        let document = Html::parse_document(&html);

        // Extract title
        let title_selector = Selector::parse("title").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_else(|| url.to_string());

        // Extract text content (remove scripts, styles, etc.)
        let content = self.extract_text_content(&document);

        // Extract links
        let links = self.extract_links(&document, url)?;

        let doc = Document::new(title, content, url.to_string());

        Ok((doc, links))
    }

    fn extract_text_content(&self, document: &Html) -> String {
        // Create a copy of the document for manipulation
        let html_string = document.html();
        let cleaned_document = Html::parse_document(&html_string);

        // Instead of regex, use DOM selectors to skip unwanted content
        let unwanted_selectors = [
            "script", "style", "nav", "footer", "aside", "noscript", "meta", "link", "head",
        ];

        // Extract text while skipping unwanted elements
        let mut content_parts = Vec::new();

        // Try to get main content areas first
        let content_selectors = vec![
            "main",
            "article",
            ".content",
            "#content",
            ".post",
            ".entry-content",
            "[role='main']",
            ".main-content",
        ];

        let mut found_main_content = false;
        for selector_str in content_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in cleaned_document.select(&selector) {
                    if !self.element_contains_unwanted_tags(&element, &unwanted_selectors) {
                        let text = self.extract_element_text(&element, &unwanted_selectors);
                        if !text.trim().is_empty() {
                            content_parts.push(text);
                            found_main_content = true;
                        }
                    }
                }
            }
        }

        // If no main content found, fall back to body
        if !found_main_content {
            if let Ok(body_selector) = Selector::parse("body") {
                for body in cleaned_document.select(&body_selector) {
                    let text = self.extract_element_text(&body, &unwanted_selectors);
                    if !text.trim().is_empty() {
                        content_parts.push(text);
                    }
                }
            }
        }

        // If still no content, try the root element
        if content_parts.is_empty() {
            let text =
                self.extract_element_text(&cleaned_document.root_element(), &unwanted_selectors);
            if !text.trim().is_empty() {
                content_parts.push(text);
            }
        }

        // Join all text parts and clean up
        let content = content_parts.join(" ");
        self.clean_extracted_text(&content)
    }

    fn element_contains_unwanted_tags(
        &self,
        element: &scraper::ElementRef,
        unwanted_selectors: &[&str],
    ) -> bool {
        for &unwanted in unwanted_selectors {
            if let Ok(selector) = Selector::parse(unwanted) {
                if element.select(&selector).next().is_some() {
                    return true;
                }
            }
        }
        false
    }

    fn extract_element_text(
        &self,
        element: &scraper::ElementRef,
        unwanted_selectors: &[&str],
    ) -> String {
        let mut text_parts = Vec::new();

        // Recursively extract text, skipping unwanted elements
        self.extract_text_recursive(element, unwanted_selectors, &mut text_parts);

        text_parts.join(" ")
    }

    fn extract_text_recursive(
        &self,
        element: &scraper::ElementRef,
        unwanted_selectors: &[&str],
        text_parts: &mut Vec<String>,
    ) {
        use scraper::Node;

        for child in element.children() {
            match child.value() {
                Node::Text(text) => {
                    let cleaned_text = text.trim();
                    if !cleaned_text.is_empty() {
                        text_parts.push(cleaned_text.to_string());
                    }
                }
                Node::Element(elem) => {
                    let tag_name = elem.name();

                    // Skip unwanted elements entirely
                    if unwanted_selectors.contains(&tag_name) {
                        continue;
                    }

                    // Recursively process other elements
                    if let Some(child_element) = scraper::ElementRef::wrap(child) {
                        self.extract_text_recursive(&child_element, unwanted_selectors, text_parts);
                    }
                }
                _ => {} // Skip comments, etc.
            }
        }
    }

    fn clean_extracted_text(&self, text: &str) -> String {
        // Clean up whitespace and normalize text
        let cleaned = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        // Remove common unwanted patterns
        let patterns_to_remove = vec![
            "Skip to main content",
            "Skip to content",
            "Jump to navigation",
            "Menu",
            "Search",
            "Toggle navigation",
        ];

        let mut result = cleaned;
        for pattern in patterns_to_remove {
            result = result.replace(pattern, "");
        }

        // Clean up extra whitespace after removals
        result.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn extract_links(&self, document: &Html, base_url: &Url) -> Result<Vec<Url>> {
        let link_selector = Selector::parse("a[href]").unwrap();
        let mut links = Vec::new();

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                // Resolve relative URLs
                if let Ok(absolute_url) = base_url.join(href) {
                    // Only include HTTP/HTTPS URLs
                    if absolute_url.scheme() == "http" || absolute_url.scheme() == "https" {
                        links.push(absolute_url);
                    }
                }
            }
        }

        Ok(links)
    }

    fn is_allowed_domain(&self, url: &Url) -> bool {
        if self.allowed_domains.is_empty() {
            return true; // No restrictions
        }

        if let Some(domain) = url.domain() {
            self.allowed_domains.contains(domain)
        } else {
            false
        }
    }
}

impl Default for WebCrawler {
    fn default() -> Self {
        Self::new()
    }
}
