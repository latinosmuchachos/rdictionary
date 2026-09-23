use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{app::AppContext, storage::Store};

pub fn render_phrase_browser(context: &mut AppContext, store: &Store, frame: &mut Frame) {
    let browser = &mut context.edit_phrase_state.browser;
    let show_search = browser.search_mode || !browser.search_pattern.lines()[0].is_empty();
    let [header, search, error, content, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(if show_search { 3 } else { 0 }),
        Constraint::Length(if browser.regex_error.is_some() { 3 } else { 0 }),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    let columns = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas::<2>(content);
    // Each phrase occupies one row; the list borders occupy two more rows.
    browser.set_page_size(usize::from(content.height.saturating_sub(2)));

    let page_number = if browser.phrase_indices.is_empty() {
        0
    } else {
        browser.page + 1
    };
    frame.render_widget(
        Paragraph::new(format!(
            "Edit dictionary / Phrases: {} of {} | Page {}/{}",
            browser.phrase_indices.len(),
            store.phrases.len(),
            page_number,
            browser.page_count(),
        ))
        .style(Style::default().fg(Color::Yellow)),
        header,
    );

    if show_search {
        browser
            .search_pattern
            .set_cursor_style(if browser.search_mode {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            });
        frame.render_widget(&browser.search_pattern, search);
    }
    if let Some(message) = &browser.regex_error {
        frame.render_widget(
            Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: false }),
            error,
        );
    }

    if browser.phrase_indices.is_empty() {
        let message = if store.phrases.is_empty() {
            "No phrases yet. Add a phrase from Edit Dictionary Menu."
        } else if browser.regex_error.is_some() {
            "Fix the search pattern or press Esc to clear it."
        } else {
            "No matching phrases. Press / to edit the search or Esc to clear it."
        };
        frame.render_widget(
            Paragraph::new(message)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
            content,
        );
    } else {
        let start = browser.page * browser.page_size;
        let end = (start + browser.page_size).min(browser.phrase_indices.len());
        let visible_indices = &browser.phrase_indices[start..end];
        for (column, title) in ["Original", "Translation"].iter().enumerate() {
            let items = visible_indices.iter().map(|&index| {
                let phrase = &store.phrases[index];
                let text = if column == 0 {
                    &phrase.original_text
                } else {
                    &phrase.translation_text
                };
                // Imported multiline text must not shift the matching rows apart.
                ListItem::new(Line::raw(text.replace(['\n', '\r'], " ")))
            });
            let list = List::new(items)
                .block(
                    Block::default()
                        .title(*title)
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
            let mut selection =
                ListState::default().with_selected(Some(browser.selected_idx - start));
            frame.render_stateful_widget(list, columns[column], &mut selection);
        }
    }

    let hint = if browser.search_mode {
        "Type regex to filter | Enter - browse results | Esc - clear search"
    } else {
        "Up/Down - phrase | Left/Right - page | Enter - select\n/ - search | Esc - clear search / back"
    };
    frame.render_widget(Paragraph::new(hint).wrap(Wrap { trim: false }), footer);
}
