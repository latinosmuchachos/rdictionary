use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::{
    app::{AddPhraseStep, AppContext}, storage::Store,
};

pub fn render_add_phrase(context: &mut AppContext, store: &Store, frame: &mut Frame) {
    let state = &context.add_phrase_state;
    let (_step_index, hint) = match state.step {
        AddPhraseStep::SelectLanguages => (
            0,
            "Tab/Up/Down - field | Left/Right - language | Enter - next | Esc - back",
        ),
        AddPhraseStep::EnterOriginal => (1, "Enter - next | Esc - languages"),
        AddPhraseStep::EnterTranslation => (2, "Enter - save phrase | Esc - original"),
    };
    let [_header, content, error, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(3),
        Constraint::Length(if state.error.is_some() { 3 } else { 0 }),
        Constraint::Length(2),
    ])
    .areas(frame.area());

    let languages = &store.languages;
    let original_language = languages
        .get(state.selected_original_lang_idx)
        .map(|language| language.name.as_str())
        .unwrap_or("No languages available");
    let translation_language = languages
        .get(state.selected_translation_lang_idx)
        .map(|language| language.name.as_str())
        .unwrap_or("No languages available");

    match state.step {
        AddPhraseStep::SelectLanguages => {
            let fields = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(content);
            for (index, title, name) in [
                (0, "Original language", original_language),
                (1, "Translation language", translation_language),
            ] {
                let style = if state.selected_language_field == index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                frame.render_widget(
                    Paragraph::new(format!("< {name} >")).style(style).block(
                        Block::default()
                            .title(title)
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(style),
                    ),
                    fields[index],
                );
            }
        }
        AddPhraseStep::EnterOriginal | AddPhraseStep::EnterTranslation => {
            let [pair, original, input, _] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(if state.step == AddPhraseStep::EnterTranslation {
                    2
                } else {
                    0
                }),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .areas(content);
            frame.render_widget(
                Paragraph::new(format!("{original_language} -> {translation_language}")),
                pair,
            );
            if state.step == AddPhraseStep::EnterTranslation {
                frame.render_widget(
                    Paragraph::new(format!("Original: {}", state.original_text.lines()[0]))
                        .wrap(Wrap { trim: false }),
                    original,
                );
                frame.render_widget(&state.translation_text, input);
            } else {
                frame.render_widget(&state.original_text, input);
            }
        }
    }

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
