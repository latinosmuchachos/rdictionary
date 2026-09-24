use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};
use regex::Regex;
use tui_textarea::{CursorMove, TextArea};

use crate::{
    menu::{EDIT_PHRASE_AFTER_SAVE_ITEMS, EDIT_PHRASE_FIELD_ITEMS},
    models::Phrase,
    storage::Store,
};

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
    EnterTranslation,
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
            search_pattern: Self::make_search_input(),
            regex_error: None,
        }
    }

    fn make_search_input() -> TextArea<'static> {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .title(" Search (regex) ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
        input.set_cursor_line_style(Style::default());
        input.set_placeholder_text("Match original or translation");
        input
    }

    pub fn selected_phrase_index(&self) -> Option<usize> {
        self.phrase_indices.get(self.selected_idx).copied()
    }

    pub fn page_count(&self) -> usize {
        self.phrase_indices.len().div_ceil(self.page_size)
    }

    pub fn set_page_size(&mut self, page_size: usize) {
        // Keep navigation valid even when the terminal has no room for a row.
        self.page_size = page_size.max(1);
        self.sync_selection();
    }

    fn sync_selection(&mut self) {
        self.selected_idx = self
            .selected_idx
            .min(self.phrase_indices.len().saturating_sub(1));
        self.page = self.selected_idx / self.page_size;
    }

    pub fn select_previous(&mut self) {
        self.selected_idx = self.selected_idx.saturating_sub(1);
        self.sync_selection();
    }

    pub fn select_next(&mut self) {
        self.selected_idx = self.selected_idx.saturating_add(1);
        self.sync_selection();
    }

    pub fn previous_page(&mut self) {
        if self.page > 0 {
            self.selected_idx -= self.page_size;
            self.sync_selection();
        }
    }

    pub fn next_page(&mut self) {
        if self.page + 1 < self.page_count() {
            self.selected_idx = self.selected_idx.saturating_add(self.page_size);
            self.sync_selection();
        }
    }

    pub fn apply_filter(&mut self, phrases: &[Phrase]) {
        let selected_phrase = self.selected_phrase_index();
        match Regex::new(&self.search_pattern.lines()[0]) {
            Ok(regex) => {
                self.phrase_indices = phrases
                    .iter()
                    .enumerate()
                    .filter(|(_, phrase)| {
                        regex.is_match(&phrase.original_text)
                            || regex.is_match(&phrase.translation_text)
                    })
                    .map(|(index, _)| index)
                    .collect();
                self.regex_error = None;
            }
            Err(error) => {
                self.phrase_indices.clear();
                self.regex_error = Some(error.to_string());
            }
        }
        self.selected_idx = selected_phrase
            .and_then(|selected| {
                self.phrase_indices
                    .iter()
                    .position(|&index| index == selected)
            })
            .unwrap_or(0);
        self.sync_selection();
    }

    pub fn clear_search(&mut self, phrases: &[Phrase]) {
        self.search_mode = false;
        self.search_pattern = Self::make_search_input();
        self.apply_filter(phrases);
    }

    pub fn xxx() {}
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
    pub field_menu: MenuState,
    pub editing_phrase: Option<Phrase>,
    pub original_field: TextArea<'static>,
    pub translation_field: TextArea<'static>,
    pub confirm_selected: bool,
    pub confirm_scroll: u16,
    pub after_save_menu: Option<MenuState>,
    pub error: Option<String>,
}

impl EditPhraseState {
    pub fn new(phrase_count: usize) -> Self {
        Self {
            browser: PhraseBrowserState::new(phrase_count),
            edit_step: EditPhraseStep::SelectField,
            field_menu: MenuState::new(EDIT_PHRASE_FIELD_ITEMS.len()),
            editing_phrase: None,
            original_field: TextArea::default(),
            translation_field: TextArea::default(),
            confirm_selected: false,
            confirm_scroll: 0,
            after_save_menu: None,
            error: None,
        }
    }

    pub fn start_editing(&mut self, phrase: &Phrase) {
        self.clear_editing();
        self.editing_phrase = Some(phrase.clone());
        self.original_field = Self::make_input(&phrase.original_text, "Original");
        self.translation_field = Self::make_input(&phrase.translation_text, "Translation");
    }

    pub fn clear_editing(&mut self) {
        self.editing_phrase = None;
        self.original_field = TextArea::default();
        self.translation_field = TextArea::default();
        self.edit_step = EditPhraseStep::SelectField;
        self.field_menu.selected = 0;
        self.confirm_selected = false;
        self.confirm_scroll = 0;
        self.after_save_menu = None;
        self.error = None;
    }

    pub fn finish_saving(&mut self, phrases: &[Phrase]) {
        self.clear_editing();
        self.browser.apply_filter(phrases);
        self.after_save_menu = Some(MenuState::new(EDIT_PHRASE_AFTER_SAVE_ITEMS.len()));
    }

    fn make_input(text: &str, title: &'static str) -> TextArea<'static> {
        let mut input = TextArea::from(text.split('\n'));
        input.set_block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        input.set_cursor_line_style(Style::default());
        input.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        input.move_cursor(CursorMove::Bottom);
        input.move_cursor(CursorMove::End);
        input
    }
}

#[derive(Debug, Default)]
pub struct SettingsLanguagesState {
    pub selected_field: usize,
    pub selecting_language: bool,
    pub lang_list_selected: usize,
    pub error: Option<String>,
    pub saved: bool,
}

#[derive(Debug)]
pub struct SettingsNumberState {
    pub input: TextArea<'static>,
    pub error: Option<String>,
    pub saved: bool,
}

impl SettingsNumberState {
    pub fn new(value: u8) -> Self {
        let mut input = TextArea::from([value.to_string()]);
        input.set_block(
            Block::default()
                .title(" Value ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        input.set_cursor_line_style(Style::default());
        input.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        input.move_cursor(CursorMove::End);
        Self {
            input,
            error: None,
            saved: false,
        }
    }

    pub fn parsed_value(&self, range: std::ops::RangeInclusive<u8>) -> Option<u8> {
        self.input.lines()[0]
            .parse::<u8>()
            .ok()
            .filter(|value| range.contains(value))
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
