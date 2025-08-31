use crate::core::{Document, Tokenizer};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostingList {
    pub term_frequency: HashMap<Uuid, usize>, // doc_id -> frequency
    pub document_frequency: usize,
}

impl PostingList {
    pub fn new() -> Self {
        Self {
            term_frequency: HashMap::new(),
            document_frequency: 0,
        }
    }

    pub fn add_document(&mut self, doc_id: Uuid, frequency: usize) {
        if self.term_frequency.insert(doc_id, frequency).is_none() {
            self.document_frequency += 1;
        }
    }

    pub fn remove_document(&mut self, doc_id: &Uuid) {
        if self.term_frequency.remove(doc_id).is_some() {
            self.document_frequency -= 1;
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InvertedIndex {
    pub index: HashMap<String, PostingList>,
    pub documents: HashMap<Uuid, Document>,
    pub total_documents: usize,
    #[serde(skip)]
    tokenizer: Tokenizer,
}

impl<'de> Deserialize<'de> for InvertedIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct InvertedIndexData {
            index: HashMap<String, PostingList>,
            documents: HashMap<Uuid, Document>,
            total_documents: usize,
        }

        let data = InvertedIndexData::deserialize(deserializer)?;

        Ok(InvertedIndex {
            index: data.index,
            documents: data.documents,
            total_documents: data.total_documents,
            tokenizer: Tokenizer::new(), // Always create a fresh tokenizer
        })
    }
}

impl InvertedIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            documents: HashMap::new(),
            total_documents: 0,
            tokenizer: Tokenizer::new(),
        }
    }

    pub fn add_document(&mut self, document: Document) {
        let doc_id = document.id;
        let tokens = self.tokenizer.tokenize(&document.get_text());

        // Count term frequencies in this document
        let mut term_freq: HashMap<String, usize> = HashMap::new();
        for token in tokens {
            *term_freq.entry(token).or_insert(0) += 1;
        }

        // Add to inverted index
        for (term, freq) in term_freq {
            let posting_list = self.index.entry(term).or_insert_with(PostingList::new);
            posting_list.add_document(doc_id, freq);
        }

        self.documents.insert(doc_id, document);
        self.total_documents += 1;
    }

    pub fn remove_document(&mut self, doc_id: &Uuid) -> Option<Document> {
        if let Some(document) = self.documents.remove(doc_id) {
            let tokens = self.tokenizer.tokenize(&document.get_text());
            let unique_tokens: HashSet<String> = tokens.into_iter().collect();

            for token in unique_tokens {
                if let Some(posting_list) = self.index.get_mut(&token) {
                    posting_list.remove_document(doc_id);
                    if posting_list.document_frequency == 0 {
                        self.index.remove(&token);
                    }
                }
            }

            self.total_documents -= 1;
            Some(document)
        } else {
            None
        }
    }

    pub fn search(&self, query: &str) -> Vec<(Uuid, f64)> {
        let query_tokens = self.tokenizer.tokenize(query);
        if query_tokens.is_empty() {
            return Vec::new();
        }

        let mut scores: HashMap<Uuid, f64> = HashMap::new();

        for token in query_tokens {
            if let Some(posting_list) = self.index.get(&token) {
                let idf = self.calculate_idf(posting_list.document_frequency);

                for (doc_id, tf) in &posting_list.term_frequency {
                    // Fixed TF calculation: Use log normalization with +1 to avoid zero
                    let tf_score = if *tf > 0 {
                        1.0 + (*tf as f64).ln() // This ensures tf_score > 1.0
                    } else {
                        0.0
                    };

                    let tf_idf = tf_score * idf;
                    *scores.entry(*doc_id).or_insert(0.0) += tf_idf;
                }
            }
        }

        let mut results: Vec<(Uuid, f64)> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results
    }

    fn calculate_idf(&self, document_frequency: usize) -> f64 {
        if document_frequency == 0 {
            0.0
        } else {
            // Smooth IDF to give some weight even to common terms
            let smooth_idf =
                ((self.total_documents as f64 + 1.0) / (document_frequency as f64 + 1.0)).ln();
            smooth_idf + 1.0 // Add base score to avoid complete zeros
        }
    }

    pub fn get_document(&self, doc_id: &Uuid) -> Option<&Document> {
        self.documents.get(doc_id)
    }

    pub fn get_all_documents(&self) -> Vec<&Document> {
        self.documents.values().collect()
    }
}

impl Default for InvertedIndex {
    fn default() -> Self {
        Self::new()
    }
}
