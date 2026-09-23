use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::{
    app::{AppContext, EditPhraseStep},
    menu::EDIT_PHRASE_FIELD_ITEMS,
    storage::Store,
};

use super::render_menu_in_area;

pub fn render_edit_phrase(context: &mut AppContext, store: &Store, frame: &mut Frame) {
    let state = &context.edit_phrase_state;
    let Some(phrase) = &state.editing_phrase else {
        frame.render_widget(
            Paragraph::new("No phrase selected. Esc - back"),
            frame.area(),
        );
        return;
    };
    let [header, content, error, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(if state.error.is_some() { 2 } else { 0 }),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    let language_name = |id| {
        store
            .languages
            .iter()
            .find(|language| language.id == id)
            .map(|language| language.name.as_str())
            .unwrap_or("Unknown language")
    };
    frame.render_widget(
        Paragraph::new(format!(
            "{} -> {}",
            language_name(phrase.original_language_id),
            language_name(phrase.translation_language_id),
        ))
        .block(
            Block::default()
                .title(format!(" Edit phrase #{} ", phrase.id))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        ),
        header,
    );

    let hint = match state.edit_step {
        EditPhraseStep::SelectField => {
            let [original, translation, menu] = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .areas(content);
            render_preview(
                frame,
                original,
                "Original",
                &state.original_field.lines().join("\n"),
            );
            render_preview(
                frame,
                translation,
                "Translation",
                &state.translation_field.lines().join("\n"),
            );
            render_menu_in_area(
                frame,
                menu,
                " What would you like to edit? ",
                "",
                &EDIT_PHRASE_FIELD_ITEMS,
                state.field_menu.selected,
            );
            "Up/Down - select | Enter - edit | Esc - cancel and return to phrases"
        }
        EditPhraseStep::EditOriginal => {
            let [preview, input] =
                Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(content);
            let translation = if state.field_menu.selected == 2 {
                state.translation_field.lines().join("\n")
            } else {
                phrase.translation_text.clone()
            };
            render_preview(frame, preview, "Translation", &translation);
            frame.render_widget(&state.original_field, input);
            if state.field_menu.selected == 2 {
                "Editing original (1/2) | Enter - translation | Esc - choose field"
            } else {
                "Editing original | Enter - continue | Esc - choose field"
            }
        }
        EditPhraseStep::EditTranslation => {
            let [preview, input] =
                Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(content);
            let original = if state.field_menu.selected == 2 {
                state.original_field.lines().join("\n")
            } else {
                phrase.original_text.clone()
            };
            render_preview(frame, preview, "Original", &original);
            frame.render_widget(&state.translation_field, input);
            if state.field_menu.selected == 2 {
                "Editing translation (2/2) | Enter - continue | Esc - original"
            } else {
                "Editing translation | Enter - continue | Esc - choose field"
            }
        }
    };
    if let Some(message) = &state.error {
        frame.render_widget(
            Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: false }),
            error,
        );
    }
    frame.render_widget(Paragraph::new(hint).wrap(Wrap { trim: false }), footer);
}

fn render_preview(frame: &mut Frame, area: Rect, title: &str, text: &str) {
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}
