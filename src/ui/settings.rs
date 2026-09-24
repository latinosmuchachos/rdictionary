use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{app::AppContext, menu::SETTINGS_MENU_ITEMS, storage::Store};

use super::render_menu;

pub fn render_settings_menu(context: &mut AppContext, _store: &Store, frame: &mut Frame) {
    render_menu(
        frame,
        " Settings ",
        " Up/Down - select | Enter - open | Esc - dictionary menu ",
        &SETTINGS_MENU_ITEMS,
        context.settings_menu_state.selected,
    );
}

pub fn render_settings_languages(context: &mut AppContext, store: &Store, frame: &mut Frame) {
    let state = &context.settings_languages_state;
    let [header, content, status, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(if state.error.is_some() { 3 } else { 1 }),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    frame.render_widget(
        Paragraph::new("Default languages").style(Style::default().fg(Color::Yellow)),
        header,
    );
    let [original, translation, _] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .areas(content);
    for (index, title, id, area) in [
        (
            0,
            "Original language",
            store.settings.default_original_language_id,
            original,
        ),
        (
            1,
            "Translation language",
            store.settings.default_translation_language_id,
            translation,
        ),
    ] {
        let name = store
            .languages
            .iter()
            .find(|language| language.id == id)
            .map(|language| language.name.as_str())
            .unwrap_or("Unknown language");
        let style = if state.selected_field == index {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        frame.render_widget(
            Paragraph::new(name).style(style).block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(style),
            ),
            area,
        );
    }
    render_status(frame, status, state.error.as_deref(), state.saved);
    let hint = if state.selecting_language {
        "Up/Down - language | Enter - save | Esc - cancel selection"
    } else {
        "Up/Down - field | Enter - choose language | Esc - settings menu"
    };
    frame.render_widget(Paragraph::new(hint).wrap(Wrap { trim: false }), footer);

    if state.selecting_language {
        let width = content.width.min(60);
        let height = store
            .languages
            .len()
            .saturating_add(2)
            .min(usize::from(content.height)) as u16;
        let popup = Rect::new(
            content.x + (content.width - width) / 2,
            content.y + (content.height - height) / 2,
            width,
            height,
        );
        let title = if state.selected_field == 0 {
            " Original language "
        } else {
            " Translation language "
        };
        let list = List::new(
            store
                .languages
                .iter()
                .map(|language| ListItem::new(language.name.as_str())),
        )
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
        let mut selection = ListState::default().with_selected(Some(state.lang_list_selected));
        frame.render_widget(Clear, popup);
        frame.render_stateful_widget(list, popup, &mut selection);
    }
}

pub fn render_settings_attempts(context: &mut AppContext, store: &Store, frame: &mut Frame) {
    let state = &context.settings_attempts_state;
    let [header, input, description, status, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(if state.error.is_some() { 3 } else { 1 }),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    frame.render_widget(
        Paragraph::new("Consecutive correct answers").style(Style::default().fg(Color::Yellow)),
        header,
    );
    frame.render_widget(&state.input, input);
    frame.render_widget(
        Paragraph::new(format!(
            "Saved value: {}\nNumber of correct answers in a row required to advance a phrase.\nApplies to new phrases. Existing phrases keep their current value.",
            store.settings.needed_attempts,
        )).wrap(Wrap { trim: false }),
        description,
    );
    render_status(frame, status, state.error.as_deref(), state.saved);
    frame.render_widget(
        Paragraph::new(
            "Enter - save | Ctrl+A - select all | Esc - settings menu (discard unsaved input)",
        )
        .wrap(Wrap { trim: false }),
        footer,
    );
}

fn render_status(frame: &mut Frame, area: Rect, error: Option<&str>, saved: bool) {
    let (message, color) = match error {
        Some(message) => (message, Color::Red),
        None if saved => ("Settings saved.", Color::Green),
        None => ("", Color::Reset),
    };
    frame.render_widget(
        Paragraph::new(message)
            .style(Style::default().fg(color))
            .wrap(Wrap { trim: false }),
        area,
    );
}
