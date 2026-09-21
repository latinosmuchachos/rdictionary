use color_eyre::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use tui_textarea::Input;

use crate::app::{App, AppInputMode, AppPage, TranslationContext, TranslationMode};
use crate::ui::{
    render_edit_dictionary_menu, render_how_many_will_translate, render_main_menu,
    render_placeholder_page, render_translate_menu, render_translate_word,
};
// use crate::events::EventHandler;

pub struct Tui {}

impl Tui {
    pub fn tick(app: &mut App) -> Result<()> {
        Self::draw_current_page(app)?;
        Self::handle_event(app)?;
        Ok(())
    }

    fn draw_current_page(app: &mut App) -> Result<()> {
        let ui_render_func = match app.context.current_page {
            AppPage::MainMenu => render_main_menu,
            AppPage::TranslationMenu => render_translate_menu,
            AppPage::QuestionHowMuchWords => render_how_many_will_translate,
            AppPage::DoTranslate => render_translate_word,
            AppPage::EditDictionaryMenu => render_edit_dictionary_menu,
            _ => render_placeholder_page,
        };
        app.terminal
            .draw(|frame| ui_render_func(&mut app.context, frame))?;
        Ok(())
    }

    fn handle_event(app: &mut App) -> Result<()> {
        match app.context.input_mode {
            AppInputMode::Key => Self::handle_key_event(app)?,
            AppInputMode::Text => Self::handle_input_text(app)?,
        }
        Ok(())
    }

    fn handle_input_text(app: &mut App) -> Result<()> {
        // TODO: change from if let to match
        if let AppPage::QuestionHowMuchWords = app.context.current_page {
            Self::handle_press_key_event_on_question_how_much_words(app, app.events.next()?)
        }
        Ok(())
    }

    fn handle_key_event(app: &mut App) -> Result<()> {
        if let KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state,
        } = app.events.next()?
        {
            Self::handle_press_key_event(app, code, modifiers, state);
        };
        Ok(())
    }

    fn handle_press_key_event(
        app: &mut App,
        code: KeyCode,
        _modifiers: KeyModifiers,
        _state: KeyEventState,
    ) {
        match code {
            KeyCode::Char('q') => app.set_should_exit(),
            _ => {
                match app.context.current_page {
                    AppPage::MainMenu => Self::handle_press_key_event_on_main_menu(app, code),
                    AppPage::TranslationMenu => {
                        Self::handle_press_key_event_on_translation_menu(app, code)
                    }
                    // AppPage::QuestionHowMuchWords => Self::handle_press_key_event_on_question_how_much_words(app, code),
                    AppPage::DoTranslate => {
                        Self::handle_press_key_event_on_do_translate_page(app, code)
                    }
                    AppPage::EditDictionaryMenu => {
                        Self::handle_press_key_event_on_edit_dictionary_menu(app, code)
                    }
                    AppPage::AddPhrase | AppPage::EditPhraseBrowser | AppPage::SettingsMenu
                        if code == KeyCode::Esc =>
                    {
                        // Temporary navigation until these pages are implemented.
                        app.context.current_page = AppPage::EditDictionaryMenu;
                    }
                    _ => {}
                }
            }
        };
    }

    fn handle_press_key_event_on_main_menu(app: &mut App, code: KeyCode) {
        match code {
            KeyCode::Char('q') => app.set_should_exit(),
            KeyCode::Char('t') => app.context.current_page = AppPage::TranslationMenu,
            KeyCode::Char('e') => app.context.current_page = AppPage::EditDictionaryMenu,
            _ => {}
        }
    }

    fn handle_press_key_event_on_translation_menu(app: &mut App, code: KeyCode) {
        match code {
            KeyCode::Char('q') => app.set_should_exit(),
            KeyCode::Esc => app.context.current_page = AppPage::MainMenu,
            KeyCode::Char('d') => {
                app.context.translation_mode = Some(TranslationMode::Daily);
                app.context.current_page = AppPage::QuestionHowMuchWords;
                app.context.input_mode = AppInputMode::Text;
            }
            KeyCode::Char('w') => {
                app.context.translation_mode = Some(TranslationMode::Weekly);
                app.context.current_page = AppPage::QuestionHowMuchWords;
                app.context.input_mode = AppInputMode::Text;
            }
            KeyCode::Char('m') => {
                app.context.translation_mode = Some(TranslationMode::Monthly);
                app.context.current_page = AppPage::QuestionHowMuchWords;
                app.context.input_mode = AppInputMode::Text;
            }
            _ => {}
        }
    }

    fn handle_press_key_event_on_question_how_much_words(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                if let Some(n) = app.context.parsed_count_from_input() {
                    app.context.translation_context =
                        Some(TranslationContext { expected_count: n });
                    app.context.end_input(AppPage::DoTranslate);
                }
            }
            KeyCode::Esc => {
                app.context.clear_input();
                app.context.end_input(AppPage::TranslationMenu);
            }
            KeyCode::Char(c) if !c.is_ascii_digit() => {}
            // блокируем ввод сверх 4 символов
            KeyCode::Char(_) if app.context.input_text.lines()[0].chars().count() >= 4 => {}
            _ => {
                app.context.input_text.input(Input::from(key));
            }
        }
    }

    fn handle_press_key_event_on_do_translate_page(app: &mut App, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                // TODO: сделать предупреждающее сообщение о выходе
                app.context.current_page = AppPage::TranslationMenu;
                app.context.translation_context = Option::None;
            }
            KeyCode::Char('q') => app.set_should_exit(),
            _ => {} // TODO: сделать ввод слов
        }
    }

    fn handle_press_key_event_on_edit_dictionary_menu(app: &mut App, code: KeyCode) {
        match code {
            KeyCode::Up => app.context.edit_dictionary_menu_state.select_previous(),
            KeyCode::Down => app.context.edit_dictionary_menu_state.select_next(),
            KeyCode::Enter => {
                app.context.current_page = match app.context.edit_dictionary_menu_state.selected {
                    0 => AppPage::AddPhrase,
                    1 => AppPage::EditPhraseBrowser,
                    2 => AppPage::SettingsMenu,
                    _ => return,
                };
            }
            KeyCode::Esc => app.context.current_page = AppPage::MainMenu,
            _ => {}
        }
    }
}
