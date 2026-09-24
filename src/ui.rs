use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

mod add_phrase;
mod edit_phrase;
mod edit_phrase_confirm;
mod phrase_browser;
mod settings;
pub use add_phrase::render_add_phrase;
pub use edit_phrase::render_edit_phrase;
pub use edit_phrase_confirm::render_edit_phrase_confirm;
pub use phrase_browser::render_phrase_browser;
pub use settings::{render_settings_attempts, render_settings_languages, render_settings_menu};

use crate::{
    app::AppContext,
    menu::{EDIT_DICTIONARY_MENU_ITEMS, MAIN_MENU_ITEMS, TRANSLATE_MENU_ITEMS},
    storage::Store,
};

// TODO: Temporary renderer for pages implemented in later steps.
pub fn render_placeholder_page(app_context: &mut AppContext, _store: &Store, frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new("This page is not implemented yet.\nEsc - back")
            .block(
                Block::default()
                    .title(format!("{:?}", app_context.current_page))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center),
        frame.area(),
    );
}

fn render_menu(frame: &mut Frame, title: &str, hint: &str, items: &[&str], selected: usize) {
    let area = frame.area();
    render_menu_in_area(frame, area, title, hint, items, selected);
}

fn render_menu_in_area(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    hint: &str,
    items: &[&str],
    selected: usize,
) {
    let menu = List::new(items.iter().copied().map(ListItem::new))
        .block(
            Block::default()
                .title(title)
                .title_bottom(hint)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Yellow))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default().with_selected(Some(selected));

    frame.render_stateful_widget(menu, area, &mut state);
}

pub fn render_main_menu(app_context: &mut AppContext, _store: &Store, frame: &mut Frame) {
    render_menu(
        frame,
        "Main Menu",
        " Up/Down - select, Enter - open, Esc - back ",
        &MAIN_MENU_ITEMS,
        app_context.main_menu_state.selected,
    );
}

pub fn render_edit_dictionary_menu(
    app_context: &mut AppContext,
    _store: &Store,
    frame: &mut Frame,
) {
    render_menu(
        frame,
        "Edit Dictionary Menu",
        " Up/Down - select, Enter - open, Esc - back ",
        &EDIT_DICTIONARY_MENU_ITEMS,
        app_context.edit_dictionary_menu_state.selected,
    );
}

pub fn render_translate_menu(app_context: &mut AppContext, _store: &Store, frame: &mut Frame) {
    render_menu(
        frame,
        "Translate Menu",
        " Up/Down - select, Enter - open, Esc - back ",
        &TRANSLATE_MENU_ITEMS,
        app_context.translate_menu_state.selected,
    );
}

pub fn render_how_many_will_translate(
    app_context: &mut AppContext,
    _store: &Store,
    frame: &mut Frame,
) {
    let outer = Block::default()
        .title(format!(" {:?} words ", app_context.translation_mode))
        .title_alignment(Alignment::Center)
        .title_bottom(" Enter - confirm, Esc - back, q")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = outer.inner(frame.area());
    frame.render_widget(outer, frame.area());

    let [question, input, hint] = Layout::vertical([
        Constraint::Length(2), // question
        Constraint::Length(3), // fields (1 line + 2 borders)
        Constraint::Min(0),    // hint/error
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new("How many words do you want to translate?")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center),
        question,
    );

    frame.render_widget(&app_context.input_text, input);

    let text = &app_context.input_text.lines()[0];
    if !text.is_empty() && app_context.parsed_count_from_input().is_none() {
        frame.render_widget(
            Paragraph::new("Enter a positive number")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center),
            hint,
        );
    }
}

pub fn render_translate_word(app_context: &mut AppContext, _store: &Store, frame: &mut Frame) {
    if let Some(translation_context) = app_context.translation_context {
        let msg = format!(
            "You should translate {} {:?} words!",
            translation_context.expected_count, app_context.translation_mode,
        );
        frame.render_widget(
            Paragraph::new(msg)
                .block(
                    Block::default()
                        .title("Main menu")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center),
            frame.area(),
        )
    } else {
        panic!(
            "Unexpected behavior in render_translate_word, context: {:?}",
            app_context
        )
    }
}
