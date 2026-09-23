use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tui_textarea::{Input, TextArea};

use crate::{
    app::{AddPhraseState, AddPhraseStep, AppContext, AppInputMode, AppPage},
    storage::Store,
};

pub(super) fn start(context: &mut AppContext, store: &Store) {
    context.add_phrase_state = AddPhraseState::from_store(store);
    context.current_page = AppPage::AddPhrase;
    context.input_mode = AppInputMode::Key;
}

pub(super) fn handle_key(context: &mut AppContext, store: &mut Store, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    let state = &mut context.add_phrase_state;
    match state.step {
        AddPhraseStep::SelectLanguages => match key.code {
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Up | KeyCode::Down => {
                state.selected_language_field = 1 - state.selected_language_field;
            }
            KeyCode::Left | KeyCode::Right if !store.languages.is_empty() => {
                let selected = if state.selected_language_field == 0 {
                    &mut state.selected_original_lang_idx
                } else {
                    &mut state.selected_translation_lang_idx
                };
                let count = store.languages.len();
                *selected = if key.code == KeyCode::Right {
                    (*selected + 1) % count
                } else if *selected == 0 {
                    count - 1
                } else {
                    *selected - 1
                };
                state.error = None;
            }
            KeyCode::Enter => {
                if store
                    .languages
                    .get(state.selected_original_lang_idx)
                    .is_some()
                    && store
                        .languages
                        .get(state.selected_translation_lang_idx)
                        .is_some()
                {
                    state.error = None;
                    state.step = AddPhraseStep::EnterOriginal;
                } else {
                    state.error = Some("No languages available. Press Esc to go back.".to_owned());
                }
            }
            KeyCode::Esc => context.current_page = AppPage::EditDictionaryMenu,
            _ => {}
        },
        AddPhraseStep::EnterOriginal => match key.code {
            KeyCode::Enter => {
                if state.original_text.lines()[0].trim().is_empty() {
                    // TODO: как обрабатывается эта ошибка?
                    state.error = Some("Enter a non-empty original phrase.".to_owned());
                } else {
                    state.error = None;
                    state.step = AddPhraseStep::EnterTranslation;
                }
            }
            KeyCode::Esc => {
                state.error = None;
                state.step = AddPhraseStep::SelectLanguages;
            }
            _ => {
                if input_single_line(&mut state.original_text, key) {
                    state.error = None;
                }
            }
        },
        AddPhraseStep::EnterTranslation => match key.code {
            KeyCode::Enter => {
                if save_phrase(state, store) {
                    context.current_page = AppPage::EditDictionaryMenu
                }
            }
            KeyCode::Esc => {
                state.error = None;
                state.step = AddPhraseStep::EnterOriginal;
            }
            _ => {
                if input_single_line(&mut state.translation_text, key) {
                    state.error = None;
                }
            }
        },
    }

    context.input_mode = if context.current_page == AppPage::AddPhrase
        && matches!(
            state.step,
            AddPhraseStep::EnterOriginal | AddPhraseStep::EnterTranslation
        ) {
        AppInputMode::Text
    } else {
        AppInputMode::Key
    };
}

fn input_single_line(input: &mut TextArea<'static>, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter => false,
        _ => input.input(Input::from(key)),
    }
}

fn save_phrase(state: &mut AddPhraseState, store: &mut Store) -> bool {
    let original = state.original_text.lines()[0].trim();
    let translation = state.translation_text.lines()[0].trim();
    if original.is_empty() || translation.is_empty() {
        state.error = Some("Both the original phrase and its translation are required.".to_owned());
        return false;
    }

    let Some(original_language) = store.languages.get(state.selected_original_lang_idx) else {
        state.error = Some("Select an available original language.".to_owned());
        return false;
    };
    let Some(translation_language) = store.languages.get(state.selected_translation_lang_idx)
    else {
        state.error = Some("Select an available translation language.".to_owned());
        return false;
    };

    let original_language_id = original_language.id;
    let translation_language_id = translation_language.id;
    match store.add_phrase(
        original_language.id,
        translation_language.id,
        original,
        translation,
    ) {
        Ok(_) => {
            tracing::debug!(
                original_language_id = original_language_id,
                translation_language_id = translation_language_id,
                original_phrase = original,
                translation_phrase = translation,
                "Saved phrase"
            );
            true
        }
        Err(error) => {
            state.error = Some(format!("Could not save phrase: {error:#}"));
            false
        }
    }
}
