use crate::app::{App, AppContext, AppPage};
use crate::storage::Store;
use crate::ui::{
    render_add_phrase, render_edit_dictionary_menu, render_edit_phrase, render_edit_phrase_confirm,
    render_how_many_will_translate, render_main_menu, render_phrase_browser,
    render_settings_attempts, render_settings_languages, render_settings_menu,
    render_translate_menu, render_translate_word, render_translation_result,
};
use color_eyre::Result;
use ratatui::Frame;
use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
};

mod add_phrase;
mod edit_phrase;
mod edit_phrase_confirm;
mod phrase_browser;
mod settings;
mod translation;

type PageRenderer = fn(&mut AppContext, &Store, &mut Frame);

pub struct Tui {}

// TODO: разделить handlers по категориям и разным файлам (как это сделано с app_phrase)
impl Tui {
    pub fn tick(app: &mut App) -> Result<()> {
        Self::draw_current_page(app)?;
        Self::handle_event(app)?;
        Ok(())
    }

    fn draw_current_page(app: &mut App) -> Result<()> {
        let (ui_render_func, renderer): (PageRenderer, &str) = match app.context.current_page {
            AppPage::MainMenu => (render_main_menu, "render_main_menu"),
            AppPage::TranslationMenu => (render_translate_menu, "render_translate_menu"),
            AppPage::QuestionHowMuchWords => (
                render_how_many_will_translate,
                "render_how_many_will_translate",
            ),
            AppPage::DoTranslate => (render_translate_word, "render_translate_word"),
            AppPage::TranslationResult => (render_translation_result, "render_translation_result"),
            AppPage::EditDictionaryMenu => {
                (render_edit_dictionary_menu, "render_edit_dictionary_menu")
            }
            AppPage::AddPhrase => (render_add_phrase, "render_add_phrase"),
            AppPage::EditPhraseBrowser => (render_phrase_browser, "render_phrase_browser"),
            AppPage::EditPhraseField => (render_edit_phrase, "render_edit_phrase"),
            AppPage::EditPhraseConfirm => {
                (render_edit_phrase_confirm, "render_edit_phrase_confirm")
            }
            AppPage::SettingsMenu => (render_settings_menu, "render_settings_menu"),
            AppPage::SettingsLanguages => (render_settings_languages, "render_settings_languages"),
            AppPage::SettingsAttempts => (render_settings_attempts, "render_settings_attempts"),
        };
        tracing::debug!(
            renderer = renderer,
            context = ?app.context,
            "Drawing page"
        );
        app.terminal
            .draw(|frame| ui_render_func(&mut app.context, &app.store, frame))?;

        Ok(())
    }

    fn handle_event(app: &mut App) -> Result<()> {
        let Event::Key(key) = app.events.next()? else {
            // A resize wakes the loop so the next draw recalculates the layout.
            return Ok(());
        };
        match app.context.current_page {
            AppPage::TranslationMenu
            | AppPage::QuestionHowMuchWords
            | AppPage::DoTranslate
            | AppPage::TranslationResult => {
                translation::handle_key(&mut app.context, &mut app.store, key);
                return Ok(());
            }
            AppPage::AddPhrase => {
                add_phrase::handle_key(&mut app.context, &mut app.store, key);
                return Ok(());
            }
            AppPage::EditPhraseBrowser => {
                phrase_browser::handle_key(&mut app.context, &app.store, key);
                return Ok(());
            }
            AppPage::EditPhraseField => {
                edit_phrase::handle_key(&mut app.context, key);
                return Ok(());
            }
            AppPage::EditPhraseConfirm => {
                edit_phrase_confirm::handle_key(&mut app.context, &mut app.store, key);
                return Ok(());
            }
            AppPage::SettingsMenu | AppPage::SettingsLanguages | AppPage::SettingsAttempts => {
                settings::handle_key(&mut app.context, &mut app.store, key);
                return Ok(());
            }
            _ => {}
        }
        tracing::debug!(
            input_mode = ?app.context.input_mode,
            "Handling event"
        );
        Self::handle_key_event(app, key);
        Ok(())
    }

    fn handle_key_event(app: &mut App, key: KeyEvent) {
        if let KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state,
        } = key
        {
            Self::handle_press_key_event(app, code, modifiers, state);
        };
    }

    fn handle_press_key_event(
        app: &mut App,
        code: KeyCode,
        _modifiers: KeyModifiers,
        _state: KeyEventState,
    ) {
        tracing::debug!(
            current_page = ?app.context.current_page,
            "Handle press key event"
        );
        match app.context.current_page {
            AppPage::MainMenu => Self::handle_press_key_event_on_main_menu(app, code),
            AppPage::EditDictionaryMenu => {
                Self::handle_press_key_event_on_edit_dictionary_menu(app, code)
            }
            _ => {}
        };
    }

    fn handle_press_key_event_on_main_menu(app: &mut App, code: KeyCode) {
        tracing::debug!(
            code = ?code,
            "Handle press key event on main menu"
        );
        match code {
            KeyCode::Up => app.context.main_menu_state.select_previous(),
            KeyCode::Down => app.context.main_menu_state.select_next(),
            KeyCode::Enter => {
                match app.context.main_menu_state.selected {
                    0 => app.context.current_page = AppPage::TranslationMenu,
                    1 => app.context.current_page = AppPage::EditDictionaryMenu,
                    2 => app.set_should_exit(),
                    _ => {}
                };
            }
            _ => {}
        }
    }

    fn handle_press_key_event_on_edit_dictionary_menu(app: &mut App, code: KeyCode) {
        tracing::debug!(
            code = ?code,
            "Handle press key event on edit dictionary menu"
        );
        match code {
            KeyCode::Up => app.context.edit_dictionary_menu_state.select_previous(),
            KeyCode::Down => app.context.edit_dictionary_menu_state.select_next(),
            KeyCode::Enter => match app.context.edit_dictionary_menu_state.selected {
                0 => add_phrase::start(&mut app.context, &app.store),
                1 => phrase_browser::start(&mut app.context, &app.store),
                2 => settings::start(&mut app.context),
                _ => {}
            },
            KeyCode::Esc => app.context.current_page = AppPage::MainMenu,
            _ => {}
        }
    }
}
