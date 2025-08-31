use crate::core::{Document, InvertedIndex};

#[derive(Debug)]
pub struct SearchResult {
    pub document: Document,
    pub score: f64,
    pub snippet: String,
}

pub struct TfIdfRanker;

impl TfIdfRanker {
    pub fn rank_documents(
        index: &InvertedIndex,
        query: &str,
        limit: Option<usize>,
    ) -> Vec<SearchResult> {
        let scored_docs = index.search(query);
        let limit = limit.unwrap_or(10);

        scored_docs
            .into_iter()
            .take(limit)
            .filter_map(|(doc_id, score)| {
                index.get_document(&doc_id).map(|doc| {
                    let snippet = Self::generate_snippet(&doc.content, query, 150);
                    SearchResult {
                        document: doc.clone(),
                        score,
                        snippet,
                    }
                })
            })
            .collect()
    }

    fn generate_snippet(content: &str, query: &str, max_length: usize) -> String {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let words: Vec<&str> = content.split_whitespace().collect();

        if words.is_empty() {
            return String::new();
        }

        // Find the first occurrence of any query word
        let mut start_index = 0;
        for (i, word) in words.iter().enumerate() {
            if query_words
                .iter()
                .any(|q| word.to_lowercase().contains(&q.to_lowercase()))
            {
                start_index = i.saturating_sub(10); // Start 10 words before
                break;
            }
        }

        let mut snippet = String::new();
        let mut length = 0;

        for word in words.iter().skip(start_index) {
            if length + word.len() + 1 > max_length {
                break;
            }

            if !snippet.is_empty() {
                snippet.push(' ');
                length += 1;
            }

            snippet.push_str(word);
            length += word.len();
        }

        if start_index > 0 {
            snippet = format!("...{}", snippet);
        }

        if length >= max_length {
            snippet.push_str("...");
        }

        snippet
    }
}
