use tui_textarea::TextArea;

use crate::models::Phrase;

use super::TranslationMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddPhraseStep {
    SelectLanguages,
    EnterOriginal,
    EnterTranslation,
    AfterSave,
}

#[derive(Debug)]
pub struct AddPhraseState {
    pub step: AddPhraseStep,
    pub selected_original_lang_idx: usize,
    pub selected_translation_lang_idx: usize,
    pub original_text: TextArea<'static>,
    pub translation_text: TextArea<'static>,
    pub after_save_selected: usize,
}

impl AddPhraseState {
    pub fn new(selected_original_lang_idx: usize, selected_translation_lang_idx: usize) -> Self {
        Self {
            step: AddPhraseStep::SelectLanguages,
            selected_original_lang_idx,
            selected_translation_lang_idx,
            original_text: TextArea::default(),
            translation_text: TextArea::default(),
            after_save_selected: 0,
        }
    }
}

#[derive(Debug)]
pub struct PhraseBrowserState {
    /// Indices of phrases in `Store::phrases` that match the active filter.
    pub phrase_indices: Vec<usize>,
    pub selected_idx: usize,
    pub page: usize,
    pub page_size: usize,
    pub search_mode: bool,
    pub search_pattern: TextArea<'static>,
    pub regex_error: Option<String>,
}

impl PhraseBrowserState {
    pub fn new(phrase_count: usize) -> Self {
        Self {
            phrase_indices: (0..phrase_count).collect(),
            selected_idx: 0,
            page: 0,
            page_size: 1,
            search_mode: false,
            search_pattern: TextArea::default(),
            regex_error: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditPhraseStep {
    SelectField,
    EditOriginal,
    EditTranslation,
}

#[derive(Debug)]
pub struct EditPhraseState {
    pub browser: PhraseBrowserState,
    pub edit_step: EditPhraseStep,
    pub selected_field: usize,
    pub editing_phrase: Option<Phrase>,
    pub original_field: TextArea<'static>,
    pub translation_field: TextArea<'static>,
    pub confirm_selected: bool,
}

impl EditPhraseState {
    pub fn new(phrase_count: usize) -> Self {
        Self {
            browser: PhraseBrowserState::new(phrase_count),
            edit_step: EditPhraseStep::SelectField,
            selected_field: 0,
            editing_phrase: None,
            original_field: TextArea::default(),
            translation_field: TextArea::default(),
            confirm_selected: false,
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

#[derive(Debug)]
pub struct TranslationSession {
    pub mode: TranslationMode,
    pub queue: Vec<Phrase>,
    pub current_idx: usize,
    pub unlimited: bool,
    pub stats: SessionStats,
    pub input: TextArea<'static>,
    pub last_answer_correct: Option<bool>,
}
