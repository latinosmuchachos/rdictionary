use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tui_textarea::{Input, TextArea};

use crate::app::{AppContext, AppInputMode, AppPage, EditPhraseState, EditPhraseStep};

pub(super) fn handle_key(context: &mut AppContext, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    let state = &mut context.edit_phrase_state;
    if state.editing_phrase.is_none() {
        context.current_page = AppPage::EditPhraseBrowser;
        context.input_mode = AppInputMode::Key;
        return;
    }

    match state.edit_step {
        EditPhraseStep::SelectField => match key.code {
            KeyCode::Up => state.field_menu.select_previous(),
            KeyCode::Down => state.field_menu.select_next(),
            KeyCode::Enter => {
                state.error = None;
                state.edit_step = if state.field_menu.selected == 1 {
                    EditPhraseStep::EditTranslation
                } else {
                    EditPhraseStep::EditOriginal
                };
            }
            KeyCode::Esc => {
                state.clear_editing();
                context.current_page = AppPage::EditPhraseBrowser;
            }
            _ => {}
        },
        EditPhraseStep::EditOriginal => match key.code {
            KeyCode::Enter => {
                if is_blank(&state.original_field) {
                    state.error = Some("Enter a non-empty original phrase.".to_owned());
                } else if state.field_menu.selected == 2 {
                    state.error = None;
                    state.edit_step = EditPhraseStep::EditTranslation;
                } else if prepare_confirmation(state) {
                    context.current_page = AppPage::EditPhraseConfirm;
                }
            }
            KeyCode::Esc => {
                state.error = None;
                state.edit_step = EditPhraseStep::SelectField;
            }
            _ => {
                if state.original_field.input(Input::from(key)) {
                    state.error = None;
                }
            }
        },
        EditPhraseStep::EditTranslation => match key.code {
            KeyCode::Enter => {
                if prepare_confirmation(state) {
                    context.current_page = AppPage::EditPhraseConfirm;
                }
            }
            KeyCode::Esc => {
                state.error = None;
                state.edit_step = if state.field_menu.selected == 2 {
                    EditPhraseStep::EditOriginal
                } else {
                    EditPhraseStep::SelectField
                };
            }
            _ => {
                if state.translation_field.input(Input::from(key)) {
                    state.error = None;
                }
            }
        },
    }

    context.input_mode = if context.current_page == AppPage::EditPhraseField
        && state.edit_step != EditPhraseStep::SelectField
    {
        AppInputMode::Text
    } else {
        AppInputMode::Key
    };
}

fn is_blank(input: &TextArea<'_>) -> bool {
    input.lines().iter().all(|line| line.trim().is_empty())
}

fn prepare_confirmation(state: &mut EditPhraseState) -> bool {
    let edit_original = matches!(state.field_menu.selected, 0 | 2);
    let edit_translation = matches!(state.field_menu.selected, 1 | 2);
    if edit_original && is_blank(&state.original_field) {
        state.error = Some("Enter a non-empty original phrase.".to_owned());
        state.edit_step = EditPhraseStep::EditOriginal;
        return false;
    }
    if edit_translation && is_blank(&state.translation_field) {
        state.error = Some("Enter a non-empty translation.".to_owned());
        return false;
    }

    let Some(phrase) = &mut state.editing_phrase else {
        return false;
    };
    // Only the chosen fields are copied to the draft. Store is updated after confirmation.
    if edit_original {
        phrase.original_text = state.original_field.lines().join("\n").trim().to_owned();
    }
    if edit_translation {
        phrase.translation_text = state.translation_field.lines().join("\n").trim().to_owned();
    }
    state.confirm_selected = false;
    state.confirm_scroll = 0;
    state.after_save_menu = None;
    state.error = None;
    tracing::debug!(
        phrase_id = phrase.id,
        "Prepared edited phrase for confirmation"
    );
    true
}
