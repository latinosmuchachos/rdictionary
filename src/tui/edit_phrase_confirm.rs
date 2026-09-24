use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::{
    app::{AppContext, AppInputMode, AppPage, EditPhraseStep},
    storage::Store,
};

pub(super) fn handle_key(context: &mut AppContext, store: &mut Store, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    context.input_mode = AppInputMode::Key;
    let state = &mut context.edit_phrase_state;
    if let Some(menu) = &mut state.after_save_menu {
        let destination = match key.code {
            KeyCode::Up => {
                menu.select_previous();
                None
            }
            KeyCode::Down => {
                menu.select_next();
                None
            }
            KeyCode::Enter => match menu.selected {
                0 => Some(AppPage::MainMenu),
                1 => Some(AppPage::EditDictionaryMenu),
                _ => None,
            },
            KeyCode::Esc => Some(AppPage::EditDictionaryMenu),
            _ => None,
        };
        if let Some(page) = destination {
            state.clear_editing();
            context.current_page = page;
        }
        return;
    }

    let Some(phrase) = &state.editing_phrase else {
        context.current_page = AppPage::EditPhraseBrowser;
        return;
    };
    match key.code {
        KeyCode::Left | KeyCode::Right => state.confirm_selected = !state.confirm_selected,
        KeyCode::Up => state.confirm_scroll = state.confirm_scroll.saturating_sub(1),
        KeyCode::Down => state.confirm_scroll = state.confirm_scroll.saturating_add(1),
        KeyCode::PageUp => state.confirm_scroll = state.confirm_scroll.saturating_sub(10),
        KeyCode::PageDown => state.confirm_scroll = state.confirm_scroll.saturating_add(10),
        KeyCode::Home => state.confirm_scroll = 0,
        KeyCode::End => state.confirm_scroll = u16::MAX,
        KeyCode::Esc => {
            state.error = None;
            context.current_page = AppPage::EditPhraseField;
            context.input_mode = if state.edit_step == EditPhraseStep::SelectField {
                AppInputMode::Key
            } else {
                AppInputMode::Text
            };
        }
        KeyCode::Enter if !state.confirm_selected => {
            state.clear_editing();
            state.browser.apply_filter(&store.phrases);
            context.current_page = AppPage::EditPhraseBrowser;
        }
        KeyCode::Enter => {
            // Keep the draft available if saving fails; Store rolls back its own changes.
            match store.update_phrase(phrase.clone()) {
                Ok(()) => {
                    tracing::debug!(
                        phrase_id = phrase.id,
                        original_language_id = phrase.original_language_id,
                        translation_language_id = phrase.translation_language_id,
                        original_phrase = phrase.original_text.as_str(),
                        translation_phrase = phrase.translation_text.as_str(),
                        "Updated phrase"
                    );
                    state.finish_saving(&store.phrases);
                }
                Err(error) => {
                    tracing::error!(phrase_id = phrase.id, error = %error, "Could not update phrase");
                    state.error = Some(format!("Could not save phrase: {error:#}"));
                }
            }
        }
        _ => {}
    }
}
