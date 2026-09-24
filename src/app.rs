use core::fmt;
use std::{
    io::{self, Stderr},
    panic,
};

use color_eyre::Result;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    },
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};
use tui_textarea::TextArea;

mod state;
mod translation;

pub use state::{
    AddPhraseState, AddPhraseStep, EditPhraseState, EditPhraseStep, SettingsAttemptsState,
    SettingsLanguagesState,
};
pub use translation::{TranslationMode, TranslationSession};

use crate::{
    app::state::MenuState,
    events::EventHandler,
    menu::{
        EDIT_DICTIONARY_MENU_ITEMS, MAIN_MENU_ITEMS, SETTINGS_MENU_ITEMS, TRANSLATE_MENU_ITEMS,
    },
    storage::{Store, get_data_dir},
};

pub type CrosstermTerminal = Terminal<CrosstermBackend<Stderr>>;

#[derive(Debug)]
pub enum AppInputMode {
    Key,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    MainMenu,
    TranslationMenu,
    QuestionHowMuchWords,
    DoTranslate,
    TranslationResult,
    EditDictionaryMenu,
    AddPhrase,
    EditPhraseBrowser,
    EditPhraseField,
    EditPhraseConfirm,
    SettingsMenu,
    SettingsLanguages,
    SettingsAttempts,
}

pub struct AppContext {
    pub should_quit: bool,
    pub input_mode: AppInputMode,
    pub current_page: AppPage,
    pub input_text: TextArea<'static>,
    pub main_menu_state: MenuState,
    pub translate_menu_state: MenuState,
    pub edit_dictionary_menu_state: MenuState,
    pub add_phrase_state: AddPhraseState,
    pub edit_phrase_state: EditPhraseState,
    pub settings_menu_state: MenuState,
    pub settings_languages_state: SettingsLanguagesState,
    pub settings_attempts_state: SettingsAttemptsState,
    pub translation_session: Option<TranslationSession>,
}

impl fmt::Debug for AppContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppContext")
            .field("should_quit", &self.should_quit)
            .field("input_mode", &self.input_mode)
            .field("current_page", &self.current_page)
            .field("main_menu_state", &self.main_menu_state)
            .field("translate_menu_state", &self.translate_menu_state)
            .field(
                "edit_dictionary_menu_state",
                &self.edit_dictionary_menu_state,
            )
            .field("translation_session", &self.translation_session)
            .field("settings_menu_state", &self.settings_menu_state)
            .finish_non_exhaustive()
    }
}

impl AppContext {
    pub fn new(store: &Store) -> Self {
        Self {
            should_quit: false,
            input_mode: AppInputMode::Key,
            current_page: AppPage::MainMenu,
            input_text: Self::make_words_input(),
            main_menu_state: MenuState::new(MAIN_MENU_ITEMS.len()),
            translate_menu_state: MenuState::new(TRANSLATE_MENU_ITEMS.len()),
            edit_dictionary_menu_state: MenuState::new(EDIT_DICTIONARY_MENU_ITEMS.len()),
            add_phrase_state: AddPhraseState::from_store(store),
            edit_phrase_state: EditPhraseState::new(store.phrases.len()),
            settings_menu_state: MenuState::new(SETTINGS_MENU_ITEMS.len()),
            settings_languages_state: SettingsLanguagesState::default(),
            settings_attempts_state: SettingsAttemptsState::from_store(store),
            translation_session: None,
        }
    }

    fn make_words_input() -> TextArea<'static> {
        let mut ta = TextArea::default();
        ta.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Number of words "),
        );
        ta.set_cursor_line_style(Style::default());
        ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        ta.set_placeholder_text("e.g. 10");
        ta
    }

    pub fn parsed_count_from_input(&self) -> Option<u8> {
        self.input_text.lines()[0]
            .trim()
            .parse::<u8>()
            .ok()
            .filter(|n| *n > 0)
    }

    pub fn clear_input(&mut self) {
        self.input_text = Self::make_words_input();
    }
}

#[derive(Debug)]
pub struct App {
    pub terminal: CrosstermTerminal,
    pub context: AppContext,
    pub events: EventHandler,
    pub store: Store,
}

impl App {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(std::io::stderr());
        let terminal = Terminal::new(backend)?;
        let store = Store::load(get_data_dir())?;
        let context = AppContext::new(&store);
        let events = EventHandler::new();
        Ok(App {
            terminal,
            context,
            events,
            store,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        ratatui::crossterm::terminal::enable_raw_mode()?;
        execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.clear()?;
        Ok(())
    }

    pub fn reset() -> Result<()> {
        ratatui::crossterm::terminal::disable_raw_mode()?;
        execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn set_should_exit(&mut self) {
        self.context.should_quit = true;
    }
}
