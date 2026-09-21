use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemorizingStep {
    New,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorizingContext {
    pub current_step: MemorizingStep,
    pub needed_attempts: u8,
    pub current_attempt: u8,
    pub last_attempt_time: Option<DateTime<Utc>>,
}
