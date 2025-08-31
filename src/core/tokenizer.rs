use regex::Regex;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Tokenizer {
    pub word_regex: Regex,
    stop_words: HashSet<String>,
}

impl Tokenizer {
    pub fn new() -> Self {
        let word_regex = Regex::new(r"\b[a-zA-Z]+\b").unwrap();
        let stop_words = Self::load_stop_words();

        Self {
            word_regex,
            stop_words,
        }
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let text = text.to_lowercase();

        self.word_regex
            .find_iter(&text)
            .map(|mat| mat.as_str().to_string())
            .filter(|word| word.len() > 2 && !self.stop_words.contains(word))
            .collect()
    }

    fn load_stop_words() -> HashSet<String> {
        let stop_words = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "is", "are", "was", "were", "be", "been", "have", "has", "had", "do", "does",
            "did", "will", "would", "could", "should", "this", "that", "these", "those", "i",
            "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them", "my", "your",
            "his", "her", "its", "our", "their",
        ];

        stop_words.into_iter().map(String::from).collect()
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}
