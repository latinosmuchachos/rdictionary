use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tui_textarea::{CursorMove, Input, TextArea};

use crate::{
    app::{AppContext, AppInputMode, AppPage, EditPhraseState, EditPhraseStep},
    storage::Store,
};

pub fn start(context: &mut AppContext, store: &Store) {
    context.edit_phrase_state = EditPhraseState::new(store.phrases.len());
    context.current_page = AppPage::EditPhraseBrowser;
    context.input_mode = AppInputMode::Key;
}

pub fn handle_key(context: &mut AppContext, store: &Store, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    let state = &mut context.edit_phrase_state;
    let browser = &mut state.browser;
    if browser.search_mode {
        match key.code {
            KeyCode::Esc => browser.clear_search(&store.phrases),
            KeyCode::Enter => {
                if browser.regex_error.is_none() {
                    browser.search_mode = false;
                }
            }
            _ => {
                if browser.search_pattern.input(Input::from(key)) {
                    browser.apply_filter(&store.phrases);
                }
            }
        }
    } else {
        match key.code {
            KeyCode::Up => browser.select_previous(),
            KeyCode::Down => browser.select_next(),
            KeyCode::Left => browser.previous_page(),
            KeyCode::Right => browser.next_page(),
            KeyCode::Char('/') => browser.search_mode = true,
            KeyCode::Esc => {
                if !browser.search_pattern.lines()[0].is_empty() {
                    browser.clear_search(&store.phrases);
                } else {
                    context.current_page = AppPage::EditDictionaryMenu;
                }
            }
            KeyCode::Enter => {
                if let Some(phrase) = browser
                    .selected_phrase_index()
                    .and_then(|index| store.phrases.get(index))
                {
                    state.editing_phrase = Some(phrase.clone());
                    state.original_field = TextArea::from([phrase.original_text.clone()]);
                    state.translation_field = TextArea::from([phrase.translation_text.clone()]);
                    state.original_field.move_cursor(CursorMove::End);
                    state.translation_field.move_cursor(CursorMove::End);
                    state.edit_step = EditPhraseStep::SelectField;
                    state.selected_field = 0;
                    state.confirm_selected = false;
                    context.current_page = AppPage::EditPhraseField;
                    tracing::debug!(phrase_id = phrase.id, "Selected phrase for editing");
                }
            }
            _ => {}
        }
    }

    context.input_mode = if browser.search_mode {
        AppInputMode::Text
    } else {
        AppInputMode::Key
    };
}
