use core::fmt;

use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};
use tui_textarea::TextArea;

use crate::{models::Phrase, storage::Store};

use super::TranslationMode;

#[derive(Debug)]
pub struct MenuState {
    pub selected: usize,
    pub items_count: usize,
}

impl MenuState {
    pub fn new(items_count: usize) -> Self {
        assert!(items_count > 0, "Menu must contain at least one item");

        Self {
            selected: 0,
            items_count,
        }
    }

    pub fn select_previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.items_count - 1
        } else {
            self.selected - 1
        };
    }

    pub fn select_next(&mut self) {
        self.selected = (self.selected + 1) % self.items_count;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddPhraseStep {
    SelectLanguages,
    EnterOriginal,
    EnterTranslation
}

#[derive(Debug)]
pub struct AddPhraseState {
    pub step: AddPhraseStep,
    pub selected_original_lang_idx: usize,
    pub selected_translation_lang_idx: usize,
    pub selected_language_field: usize,
    pub original_text: TextArea<'static>,
    pub translation_text: TextArea<'static>,
    pub error: Option<String>,
}

impl AddPhraseState {
    pub fn from_store(store: &Store) -> Self {
        let original = store
            .languages
            .iter()
            .position(|language| language.id == store.settings.default_original_language_id)
            .unwrap_or(0);
        let translation = store
            .languages
            .iter()
            .position(|language| language.id == store.settings.default_translation_language_id)
            .unwrap_or(0);
        Self::new(original, translation)
    }

    pub fn new(selected_original_lang_idx: usize, selected_translation_lang_idx: usize) -> Self {
        Self {
            step: AddPhraseStep::SelectLanguages,
            selected_original_lang_idx,
            selected_translation_lang_idx,
            selected_language_field: 0,
            original_text: Self::make_input("Original phrase"),
            translation_text: Self::make_input("Translation"),
            error: None,
        }
    }

    fn make_input(title: &'static str) -> TextArea<'static> {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        input.set_cursor_line_style(Style::default());
        input.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        input
    }
}

#[derive(Debug)]
pub struct PhraseBrowserState {
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

pub struct TranslationSession {
    pub mode: TranslationMode,
    pub queue: Vec<Phrase>,
    pub current_idx: usize,
    pub unlimited: bool,
    pub stats: SessionStats,
    pub input: TextArea<'static>,
    pub last_answer_correct: Option<bool>,
}

impl fmt::Debug for TranslationSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslationSession")
            .field("mode", &self.mode)
            .field("current_idx", &self.current_idx)
            .field("count_in_queue", &self.queue.len())
            .field("unlimited", &self.unlimited)
            .field("stats", &self.stats)
            .field("input", &self.input)
            .field("last_answer_correct", &self.last_answer_correct)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use crate::menu::EDIT_DICTIONARY_MENU_ITEMS;

    use super::MenuState;

    #[test]
    fn menu_navigation_uses_its_own_item_count() {
        let mut state = MenuState::new(2);
        state.select_next();
        assert_eq!(state.selected, 1);
        state.select_next();
        assert_eq!(state.selected, 0);
        state.select_previous();
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn edit_dictionary_menu_navigation_wraps() {
        let mut state = MenuState::new(EDIT_DICTIONARY_MENU_ITEMS.len());
        assert_eq!(state.selected, 0);

        state.select_previous();
        assert_eq!(state.selected, 2);

        state.select_next();
        assert_eq!(state.selected, 0);

        state.select_next();
        assert_eq!(state.selected, 1);

        state.select_previous();
        assert_eq!(state.selected, 0);
    }
}
