use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_original_language_id: u32,
    pub default_translation_language_id: u32,
    pub needed_attempts: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_original_language_id: 1,
            default_translation_language_id: 2,
            needed_attempts: 5,
        }
    }
}
