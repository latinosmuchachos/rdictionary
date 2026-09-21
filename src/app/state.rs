use tui_textarea::TextArea;

use crate::models::Phrase;

use super::TranslationMode;

// TODO: придумать, как учитывать не эти поля а непосредственно размеры MAIN_MENU_ITEMS и EDIT_DICTIONARY_MUNY_ITEMS
const MAIN_MENU_ITEMS_COUNT: usize = 3;
const EDIT_DICTIONARY_MENU_ITEMS_COUNT: usize = 3;

// TODO: сделать абстрактный класс для выбора и "отнаследовать" от него MainMenuState и EditDictionaryMenuState
#[derive(Debug, Default)]
pub struct MainMenuState {
    pub selected: usize,
}

impl MainMenuState {
    pub fn select_previous(&mut self) {
        self.selected = if self.selected == 0 {
            MAIN_MENU_ITEMS_COUNT - 1
        } else {
            self.selected - 1
        };
    }

    pub fn select_next(&mut self) {
        self.selected = (self.selected + 1) % MAIN_MENU_ITEMS_COUNT;
    }
}

#[derive(Debug, Default)]
pub struct EditDictionaryMenuState {
    pub selected: usize,
}

impl EditDictionaryMenuState {
    pub fn select_previous(&mut self) {
        self.selected = if self.selected == 0 {
            EDIT_DICTIONARY_MENU_ITEMS_COUNT - 1
        } else {
            self.selected - 1
        };
    }

    pub fn select_next(&mut self) {
        self.selected = (self.selected + 1) % EDIT_DICTIONARY_MENU_ITEMS_COUNT;
    }
}

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

#[cfg(test)]
mod tests {
    use super::EditDictionaryMenuState;

    #[test]
    fn edit_dictionary_menu_navigation_wraps() {
        let mut state = EditDictionaryMenuState::default();
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
