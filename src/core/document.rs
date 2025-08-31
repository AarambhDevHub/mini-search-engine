use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub path: String,
    pub word_count: usize,
}

impl Document {
    pub fn new(title: String, content: String, path: String) -> Self {
        let word_count = content.split_whitespace().count();

        Self {
            id: Uuid::new_v4(),
            title,
            content,
            path,
            word_count,
        }
    }

    pub fn get_text(&self) -> String {
        format!("{} {}", self.title, self.content)
    }
}
