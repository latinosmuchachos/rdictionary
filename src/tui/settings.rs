use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tui_textarea::Input;

use crate::{
    app::{AppContext, AppInputMode, AppPage, SettingsLanguagesState, SettingsNumberState},
    storage::Store,
};

pub(super) fn start(context: &mut AppContext) {
    context.settings_menu_state.selected = 0;
    context.current_page = AppPage::SettingsMenu;
    context.input_mode = AppInputMode::Key;
}

pub(super) fn handle_key(context: &mut AppContext, store: &mut Store, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    match context.current_page {
        AppPage::SettingsMenu => handle_menu(context, store, key.code),
        AppPage::SettingsLanguages => handle_languages(context, store, key.code),
        AppPage::SettingsAttempts | AppPage::SettingsReverseProbability => {
            handle_number(context, store, key)
        }
        _ => return,
    }
    context.input_mode = if matches!(
        context.current_page,
        AppPage::SettingsAttempts | AppPage::SettingsReverseProbability
    ) {
        AppInputMode::Text
    } else {
        AppInputMode::Key
    };
}

fn handle_menu(context: &mut AppContext, store: &Store, code: KeyCode) {
    match code {
        KeyCode::Up => context.settings_menu_state.select_previous(),
        KeyCode::Down => context.settings_menu_state.select_next(),
        KeyCode::Enter => match context.settings_menu_state.selected {
            0 => {
                context.settings_languages_state = SettingsLanguagesState::default();
                context.current_page = AppPage::SettingsLanguages;
            }
            1 => {
                context.settings_number_state =
                    SettingsNumberState::new(store.settings.needed_attempts);
                context.current_page = AppPage::SettingsAttempts;
            }
            2 => {
                context.settings_number_state =
                    SettingsNumberState::new(store.settings.reverse_translation_probability);
                context.current_page = AppPage::SettingsReverseProbability;
            }
            _ => {}
        },
        KeyCode::Esc => context.current_page = AppPage::EditDictionaryMenu,
        _ => {}
    }
}

fn handle_languages(context: &mut AppContext, store: &mut Store, code: KeyCode) {
    let state = &mut context.settings_languages_state;
    if !state.selecting_language {
        match code {
            KeyCode::Up | KeyCode::Down => {
                state.selected_field = 1 - state.selected_field;
                state.error = None;
                state.saved = false;
            }
            KeyCode::Enter => {
                state.saved = false;
                if store.languages.is_empty() {
                    state.error = Some("No languages available.".to_owned());
                    return;
                }
                let language_id = if state.selected_field == 0 {
                    store.settings.default_original_language_id
                } else {
                    store.settings.default_translation_language_id
                };
                state.lang_list_selected = store
                    .languages
                    .iter()
                    .position(|language| language.id == language_id)
                    .unwrap_or(0);
                state.selecting_language = true;
                state.error = None;
            }
            KeyCode::Esc => context.current_page = AppPage::SettingsMenu,
            _ => {}
        }
        return;
    }

    let count = store.languages.len();
    match code {
        KeyCode::Up | KeyCode::Down if count > 0 => {
            state.lang_list_selected = if code == KeyCode::Down {
                (state.lang_list_selected + 1) % count
            } else if state.lang_list_selected == 0 {
                count - 1
            } else {
                state.lang_list_selected - 1
            };
            state.error = None;
        }
        KeyCode::Esc => {
            state.selecting_language = false;
            state.error = None;
        }
        KeyCode::Enter => {
            let Some(language) = store.languages.get(state.lang_list_selected) else {
                state.error = Some("Select an available language.".to_owned());
                return;
            };
            let mut settings = store.settings.clone();
            if state.selected_field == 0 {
                settings.default_original_language_id = language.id;
            } else {
                settings.default_translation_language_id = language.id;
            }
            match store.update_settings(settings) {
                Ok(()) => {
                    state.selecting_language = false;
                    state.error = None;
                    state.saved = true;
                    tracing::debug!(
                        original_language_id = store.settings.default_original_language_id,
                        translation_language_id = store.settings.default_translation_language_id,
                        "Saved default languages"
                    );
                }
                Err(error) => {
                    tracing::error!(error = %error, "Could not save default languages");
                    state.error = Some(format!("Could not save settings: {error:#}"));
                }
            }
        }
        _ => {}
    }
}

fn handle_number(context: &mut AppContext, store: &mut Store, key: KeyEvent) {
    let reverse_probability = context.current_page == AppPage::SettingsReverseProbability;
    let range = if reverse_probability {
        0..=100
    } else {
        1..=255
    };
    let state = &mut context.settings_number_state;
    match key.code {
        KeyCode::Esc => context.current_page = AppPage::SettingsMenu,
        KeyCode::Enter => {
            state.saved = false;
            let Some(value) = state.parsed_value(range.clone()) else {
                state.error = Some(format!(
                    "Enter a whole number from {} to {}.",
                    range.start(),
                    range.end()
                ));
                return;
            };
            let mut settings = store.settings.clone();
            if reverse_probability {
                settings.reverse_translation_probability = value;
            } else {
                settings.needed_attempts = value;
            }
            match store.update_settings(settings) {
                Ok(()) => {
                    state.error = None;
                    state.saved = true;
                    tracing::debug!(page = ?context.current_page, value, "Saved numeric setting");
                }
                Err(error) => {
                    tracing::error!(error = %error, "Could not save numeric setting");
                    state.error = Some(format!("Could not save settings: {error:#}"));
                }
            }
        }
        KeyCode::Char('a') if key.modifiers == KeyModifiers::CONTROL => state.input.select_all(),
        _ => {
            // Only allow digits and editing keys; TextArea shortcuts can insert newlines.
            let allowed = match key.code {
                KeyCode::Char(character) => {
                    character.is_ascii_digit()
                        && !key
                            .modifiers
                            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                }
                KeyCode::Left
                | KeyCode::Right
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::Backspace
                | KeyCode::Delete => true,
                _ => false,
            };
            if allowed && state.input.input(Input::from(key)) {
                state.error = None;
                state.saved = false;
            }
        }
    }
}
