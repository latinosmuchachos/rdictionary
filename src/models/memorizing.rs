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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FailuresByStep {
    pub new: u32,
    pub daily: u32,
    pub weekly: u32,
    pub monthly: u32,
}

impl FailuresByStep {
    pub fn count(&self, step: MemorizingStep) -> u32 {
        match step {
            MemorizingStep::New => self.new,
            MemorizingStep::Daily => self.daily,
            MemorizingStep::Weekly => self.weekly,
            MemorizingStep::Monthly => self.monthly,
        }
    }

    pub fn increment(&mut self, step: MemorizingStep) {
        let count = match step {
            MemorizingStep::New => &mut self.new,
            MemorizingStep::Daily => &mut self.daily,
            MemorizingStep::Weekly => &mut self.weekly,
            MemorizingStep::Monthly => &mut self.monthly,
        };
        *count = count.saturating_add(1);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorizingContext {
    pub current_step: MemorizingStep,
    pub needed_attempts: u8,
    pub current_attempt: u8,
    pub last_attempt_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub failures: FailuresByStep,
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

            self.failures.increment(self.current_step);
            self.current_attempt = 0;
            self.current_step = self.current_step.prev_clamped();
        }
        self.last_attempt_time = Some(now);
    }
}
