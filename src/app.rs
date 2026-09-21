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

pub use state::{AddPhraseState, EditDictionaryMenuState, EditPhraseState, TranslationSession};

use crate::{
    app::state::{MainMenuState, TranslateMenuState}, 
    events::EventHandler, 
    storage::{Store, get_data_dir},
};

pub type CrosstermTerminal = Terminal<CrosstermBackend<Stderr>>;

#[derive(Debug)]
pub enum AppInputMode {
    Key,
    Text,
}

#[derive(Debug, Copy, Clone)]
pub struct TranslationContext {
    pub expected_count: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationMode {
    Daily,
    Weekly,
    Monthly,
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

#[derive(Debug)]
pub struct AppContext {
    pub should_quit: bool,
    pub input_mode: AppInputMode,
    pub translation_mode: Option<TranslationMode>,
    pub current_page: AppPage,
    pub input_text: TextArea<'static>,
    pub translation_context: Option<TranslationContext>,
    pub main_menu_state: MainMenuState,
    pub translate_menu_state: TranslateMenuState,
    pub edit_dictionary_menu_state: EditDictionaryMenuState,
    pub add_phrase_state: AddPhraseState,
    pub edit_phrase_state: EditPhraseState,
    pub translation_session: Option<TranslationSession>,
}

impl AppContext {
    fn new(store: &Store) -> Self {
        let original_language_idx = store
            .languages
            .iter()
            .position(|language| language.id == store.settings.default_original_language_id)
            .unwrap_or(0);
        let translation_language_idx = store
            .languages
            .iter()
            .position(|language| language.id == store.settings.default_translation_language_id)
            .unwrap_or(0);

        Self {
            should_quit: false,
            input_mode: AppInputMode::Key,
            translation_mode: None,
            current_page: AppPage::MainMenu,
            input_text: Self::make_words_input(),
            translation_context: None,
            main_menu_state: MainMenuState::default(),
            translate_menu_state: TranslateMenuState::default(),
            edit_dictionary_menu_state: EditDictionaryMenuState::default(),
            add_phrase_state: AddPhraseState::new(original_language_idx, translation_language_idx),
            edit_phrase_state: EditPhraseState::new(store.phrases.len()),
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
        self.input_text.select_all();
        self.input_text.cut();
    }

    pub fn end_input(&mut self, next_page: AppPage) {
        self.current_page = next_page;
        self.input_mode = AppInputMode::Key;
        self.clear_input();
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
