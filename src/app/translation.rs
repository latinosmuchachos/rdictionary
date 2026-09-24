use std::{cmp::Reverse, fmt};

use chrono::{Datelike, Duration, NaiveDate};
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};
use tui_textarea::TextArea;

use crate::{
    models::{MemorizingStep, Phrase},
    storage::Store,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationMode {
    New,
    Daily,
    Weekly,
    Monthly,
}

impl TranslationMode {
    pub fn includes(self, phrase: &Phrase, today: NaiveDate) -> bool {
        let step = match self {
            Self::New => MemorizingStep::New,
            Self::Daily => MemorizingStep::Daily,
            Self::Weekly => MemorizingStep::Weekly,
            Self::Monthly => MemorizingStep::Monthly,
        };
        let memorizing = &phrase.memorizing_context;
        if memorizing.current_step != step {
            return false;
        }
        let period_start = match self {
            Self::New => return true,
            Self::Daily => today,
            Self::Weekly => today
                .checked_sub_signed(Duration::days(i64::from(
                    today.weekday().num_days_from_monday(),
                )))
                .unwrap_or(NaiveDate::MIN),
            Self::Monthly => today.with_day(1).expect("every month has a first day"),
        };
        memorizing
            .last_attempt_time
            .is_none_or(|last_attempt| last_attempt.date_naive() < period_start)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationDirection {
    Forward,
    Reverse,
}

impl TranslationDirection {
    fn choose(reverse_probability: u8) -> Self {
        if rand::random_ratio(u32::from(reverse_probability), 100) {
            Self::Reverse
        } else {
            Self::Forward
        }
    }

    pub fn prompt(self, phrase: &Phrase) -> &str {
        match self {
            Self::Forward => &phrase.original_text,
            Self::Reverse => &phrase.translation_text,
        }
    }

    pub fn expected_answer(self, phrase: &Phrase) -> &str {
        match self {
            Self::Forward => &phrase.translation_text,
            Self::Reverse => &phrase.original_text,
        }
    }

    pub fn language_ids(self, phrase: &Phrase) -> (u32, u32) {
        match self {
            Self::Forward => (phrase.original_language_id, phrase.translation_language_id),
            Self::Reverse => (phrase.translation_language_id, phrase.original_language_id),
        }
    }
}

#[derive(Debug, Default)]
pub struct SessionStats {
    pub correct: u32,
    pub incorrect: u32,
    pub leveled_up: u32,
    pub leveled_down: u32,
    pub same_level: u32,
}

impl SessionStats {
    pub fn record(&mut self, correct: bool, previous: MemorizingStep, current: MemorizingStep) {
        if correct {
            self.correct += 1;
        } else {
            self.incorrect += 1;
        }
        // At the Daily/Monthly boundaries an answer may leave the level unchanged.
        if previous == current {
            self.same_level += 1;
        } else if correct {
            self.leveled_up += 1;
        } else {
            self.leveled_down += 1;
        }
    }
}

pub struct TranslationSession {
    pub mode: TranslationMode,
    pub direction: TranslationDirection,
    reverse_probability: u8,
    pub queue: Vec<Phrase>,
    pub current_idx: usize,
    pub unlimited: bool,
    pub stats: SessionStats,
    pub input: TextArea<'static>,
    pub last_answer_correct: Option<bool>,
    pub error: Option<String>,
    pub phrase_scroll: u16,
}

impl TranslationSession {
    pub fn from_store(store: &Store, mode: TranslationMode, today: NaiveDate) -> Self {
        let mut queue: Vec<Phrase> = store
            .phrases
            .iter()
            .filter(|phrase| {
                phrase.original_language_id == store.settings.default_original_language_id
                    && phrase.translation_language_id
                        == store.settings.default_translation_language_id
                    && mode.includes(phrase, today)
            })
            .cloned()
            .collect();

        queue.sort_by_key(|phrase| {
            let context = &phrase.memorizing_context;
            (
                Reverse(context.failures.count(context.current_step)),
                context.last_attempt_time,
                phrase.id,
            )
        });
        Self {
            mode,
            direction: TranslationDirection::choose(store.settings.reverse_translation_probability),
            reverse_probability: store.settings.reverse_translation_probability,
            queue,
            current_idx: 0,
            unlimited: false,
            stats: SessionStats::default(),
            input: Self::make_input(),
            last_answer_correct: None,
            error: None,
            phrase_scroll: 0,
        }
    }

    fn make_input() -> TextArea<'static> {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .title(" Your translation ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        input.set_cursor_line_style(Style::default());
        input.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        input
    }

    pub fn current_phrase(&self) -> Option<&Phrase> {
        self.queue.get(self.current_idx)
    }

    pub fn advance(&mut self) {
        self.current_idx = self.current_idx.saturating_add(1).min(self.queue.len());
        if self.current_phrase().is_some() {
            self.direction = TranslationDirection::choose(self.reverse_probability);
        }
        self.input = Self::make_input();
        self.last_answer_correct = None;
        self.error = None;
        self.phrase_scroll = 0;
    }
}

impl fmt::Debug for TranslationSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslationSession")
            .field("mode", &self.mode)
            .field("direction", &self.direction)
            .field("current_idx", &self.current_idx)
            .field("count_in_queue", &self.queue.len())
            .field("unlimited", &self.unlimited)
            .field("stats", &self.stats)
            .field("last_answer_correct", &self.last_answer_correct)
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}
