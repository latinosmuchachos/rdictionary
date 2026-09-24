use chrono::Utc;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tui_textarea::Input;

use crate::{
    app::{AppContext, AppInputMode, AppPage, TranslationMode, TranslationSession},
    storage::Store,
};

pub(super) fn handle_key(context: &mut AppContext, store: &mut Store, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    match context.current_page {
        AppPage::TranslationMenu => handle_menu(context, store, key.code),
        AppPage::QuestionHowMuchWords => handle_count(context, key),
        AppPage::DoTranslate => handle_answer(context, store, key),
        AppPage::TranslationResult => {
            if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                return_to_menu(context);
            }
        }
        _ => {}
    }
}

fn handle_menu(context: &mut AppContext, store: &Store, code: KeyCode) {
    match code {
        KeyCode::Up => context.translate_menu_state.select_previous(),
        KeyCode::Down => context.translate_menu_state.select_next(),
        KeyCode::Esc => context.current_page = AppPage::MainMenu,
        KeyCode::Enter => {
            let mode = match context.translate_menu_state.selected {
                0 => TranslationMode::New,
                1 => TranslationMode::Daily,
                2 => TranslationMode::Weekly,
                3 => TranslationMode::Monthly,
                _ => return,
            };
            context.translation_session = Some(TranslationSession::from_store(
                store,
                mode,
                Utc::now().date_naive(),
            ));
            context.clear_input();
            context.current_page = AppPage::QuestionHowMuchWords;
            context.input_mode = AppInputMode::Text;
        }
        _ => {}
    }
}

fn handle_count(context: &mut AppContext, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => return_to_menu(context),
        KeyCode::Enter => {
            if let Some(count) = context.parsed_count_from_input() {
                start_session(context, Some(count));
            } else if let Some(session) = &mut context.translation_session {
                session.error = Some(
                    "Enter a whole number from 1 to 255, or press u for all phrases.".to_owned(),
                );
            }
        }
        KeyCode::Char('u')
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            start_session(context, None);
        }
        KeyCode::Char('a') if key.modifiers == KeyModifiers::CONTROL => {
            context.input_text.select_all()
        }
        _ => {
            let allowed = match key.code {
                KeyCode::Char(c) => {
                    c.is_ascii_digit()
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
            if allowed
                && context.input_text.input(Input::from(key))
                && let Some(session) = &mut context.translation_session
            {
                session.error = None;
            }
        }
    }
}

fn start_session(context: &mut AppContext, limit: Option<u8>) {
    let Some(session) = &mut context.translation_session else {
        return_to_menu(context);
        return;
    };
    if let Some(count) = limit {
        session.queue.truncate(usize::from(count));
    }
    session.unlimited = limit.is_none();
    session.error = None;
    if session.queue.is_empty() {
        context.current_page = AppPage::TranslationResult;
        context.input_mode = AppInputMode::Key;
    } else {
        context.current_page = AppPage::DoTranslate;
        context.input_mode = AppInputMode::Text;
    }
    tracing::debug!(mode = ?session.mode, count = session.queue.len(), unlimited = session.unlimited, "Started translation session");
    context.clear_input();
}

fn handle_answer(context: &mut AppContext, store: &mut Store, key: KeyEvent) {
    let Some(session) = &mut context.translation_session else {
        return_to_menu(context);
        return;
    };
    match key.code {
        KeyCode::Esc => {
            context.current_page = AppPage::TranslationResult;
            context.input_mode = AppInputMode::Key;
        }
        KeyCode::PageUp => session.phrase_scroll = session.phrase_scroll.saturating_sub(5),
        KeyCode::PageDown => session.phrase_scroll = session.phrase_scroll.saturating_add(5),
        KeyCode::Char('j')
            if key.modifiers == KeyModifiers::CONTROL && session.last_answer_correct.is_none() =>
        {
            // Enter submits the answer, so provide a separate newline shortcut.
            session.input.insert_newline();
            session.error = None;
        }
        KeyCode::Enter if session.last_answer_correct.is_some() => {
            session.advance();
            if session.current_phrase().is_none() {
                context.current_page = AppPage::TranslationResult;
                context.input_mode = AppInputMode::Key;
            } else {
                context.input_mode = AppInputMode::Text;
            }
        }
        KeyCode::Enter => {
            check_answer(session, store);
            if session.last_answer_correct.is_some() {
                context.input_mode = AppInputMode::Key;
            }
        }
        _ if session.last_answer_correct.is_none() && session.input.input(Input::from(key)) => {
            session.error = None;
        }
        _ => {}
    }
}

fn check_answer(session: &mut TranslationSession, store: &mut Store) {
    let Some(phrase) = session.current_phrase() else {
        return;
    };
    let answer = session.input.lines().join("\n");
    if answer.trim().is_empty() {
        session.error = Some("Enter a translation.".to_owned());
        return;
    }
    let correct = answer.trim().to_lowercase() == phrase.translation_text.trim().to_lowercase();
    let previous_step = phrase.memorizing_context.current_step;
    let mut updated = phrase.clone();
    updated
        .memorizing_context
        .record_answer(correct, Utc::now());

    // Commit the phrase before counting the answer or advancing the session.
    match store.update_phrase(updated.clone()) {
        Ok(()) => {
            session.stats.record(
                correct,
                previous_step,
                updated.memorizing_context.current_step,
            );
            tracing::debug!(
                phrase_id = updated.id,
                correct,
                previous_step = ?previous_step,
                current_step = ?updated.memorizing_context.current_step,
                current_attempt = updated.memorizing_context.current_attempt,
                "Saved translation answer"
            );
            session.queue[session.current_idx] = updated;
            session.last_answer_correct = Some(correct);
            session.error = None;
            session.phrase_scroll = 0;
        }
        Err(error) => {
            tracing::error!(phrase_id = updated.id, error = %error, "Could not save translation answer");
            session.error = Some(format!(
                "Could not save progress: {error:#}. Enter - retry, Esc - results."
            ));
        }
    }
}

fn return_to_menu(context: &mut AppContext) {
    context.translation_session = None;
    context.clear_input();
    context.current_page = AppPage::TranslationMenu;
    context.input_mode = AppInputMode::Key;
}
