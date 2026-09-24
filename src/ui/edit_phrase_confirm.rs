use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::{app::AppContext, menu::EDIT_PHRASE_AFTER_SAVE_ITEMS, storage::Store};

use super::render_menu_in_area;

pub fn render_edit_phrase_confirm(context: &mut AppContext, store: &Store, frame: &mut Frame) {
    let state = &mut context.edit_phrase_state;
    if let Some(menu) = &state.after_save_menu {
        render_menu_in_area(
            frame,
            frame.area(),
            " Phrase saved ",
            " Up/Down - select | Enter - open | Esc - dictionary menu ",
            &EDIT_PHRASE_AFTER_SAVE_ITEMS,
            menu.selected,
        );
        return;
    }

    let Some(phrase) = &state.editing_phrase else {
        frame.render_widget(
            Paragraph::new("No phrase selected. Esc - back"),
            frame.area(),
        );
        return;
    };
    let [header, content, error, buttons, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(if state.error.is_some() { 3 } else { 0 }),
        Constraint::Length(3),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    frame.render_widget(
        Paragraph::new("Save changes to this phrase?").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        header,
    );
    let language_name = |id| {
        store
            .languages
            .iter()
            .find(|language| language.id == id)
            .map(|language| language.name.as_str())
            .unwrap_or("Unknown language")
    };
    let details = Paragraph::new(format!(
        "Original ({}):\n{}\n\nTranslation ({}):\n{}",
        language_name(phrase.original_language_id),
        phrase.original_text,
        language_name(phrase.translation_language_id),
        phrase.translation_text,
    ))
    .wrap(Wrap { trim: false });
    let block = Block::default()
        .title(format!(" Phrase #{} ", phrase.id))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let inner = block.inner(content);
    // Use the same wrapping calculation as the renderer to keep the last line reachable.
    let max_scroll = if inner.width == 0 || inner.height == 0 {
        0
    } else {
        details
            .line_count(inner.width)
            .saturating_sub(usize::from(inner.height))
    };
    state.confirm_scroll = state
        .confirm_scroll
        .min(u16::try_from(max_scroll).unwrap_or(u16::MAX));
    frame.render_widget(block, content);
    frame.render_widget(details.scroll((state.confirm_scroll, 0)), inner);

    if let Some(message) = &state.error {
        frame.render_widget(
            Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: false }),
            error,
        );
    }
    let choices = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas::<2>(buttons);
    for (index, label) in ["Yes", "No"].iter().enumerate() {
        let selected = state.confirm_selected == (index == 0);
        let style = if selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        frame.render_widget(
            Paragraph::new(*label)
                .alignment(Alignment::Center)
                .style(style)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                ),
            choices[index],
        );
    }
    frame.render_widget(
        Paragraph::new("Left/Right - Yes/No | Enter - confirm | Esc - edit\nUp/Down or PgUp/PgDn - scroll phrase")
            .wrap(Wrap { trim: false }),
        footer,
    );
}
