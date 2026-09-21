use super::MemorizingContext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phrase {
    pub id: u32,
    pub original_language_id: u32,
    pub translation_language_id: u32,
    pub original_text: String,
    pub translation_text: String,
    pub memorizing_context: MemorizingContext,
}
