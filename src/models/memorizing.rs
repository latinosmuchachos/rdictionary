use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemorizingStep {
    New,
    Daily,
    Weekly,
    Monthly,
}

impl MemorizingStep {
    pub fn next(self) -> Self {
        match self {
            Self::New => Self::Daily,
            Self::Daily => Self::Weekly,
            Self::Weekly | Self::Monthly => Self::Monthly,
        }
    }

    pub fn prev_clamped(self) -> Self {
        match self {
            Self::New => Self::New,
            Self::Daily | Self::Weekly => Self::Daily,
            Self::Monthly => Self::Weekly,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorizingContext {
    pub current_step: MemorizingStep,
    pub needed_attempts: u8,
    pub current_attempt: u8,
    pub last_attempt_time: Option<DateTime<Utc>>,
}

impl MemorizingContext {
    pub fn record_answer(&mut self, correct: bool, now: DateTime<Utc>) {
        if correct {
            self.current_attempt = self.current_attempt.saturating_add(1);
            if self.current_attempt >= self.needed_attempts {
                self.current_step = self.current_step.next();
                self.current_attempt = 0;
            }
        } else {
            self.current_attempt = 0;
            self.current_step = self.current_step.prev_clamped();
        }
        self.last_attempt_time = Some(now);
    }
}
