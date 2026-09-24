use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::{app::AppContext, storage::Store};

pub fn render_how_many_will_translate(context: &mut AppContext, store: &Store, frame: &mut Frame) {
    let Some(session) = &context.translation_session else {
        frame.render_widget(
            Paragraph::new("No active session. Esc - menu."),
            frame.area(),
        );
        return;
    };
    let [header, input, info, error, footer] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(if session.error.is_some() { 3 } else { 0 }),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    frame.render_widget(
        Paragraph::new(format!(
            "{:?} phrases | {}",
            session.mode,
            language_pair(store)
        ))
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: false }),
        header,
    );
    frame.render_widget(&context.input_text, input);
    let available = session.queue.len();
    let description = if available == 0 {
        "No phrases are due for this mode and language pair.".to_owned()
    } else {
        format!(
            "Available phrases: {available}\nReverse translation probability: {}%\nEnter a number from 1 to 255, or press u to practice all available phrases.\nIf the number is larger than the queue, the session uses all available phrases.",
            store.settings.reverse_translation_probability,
        )
    };
    frame.render_widget(Paragraph::new(description).wrap(Wrap { trim: false }), info);
    if let Some(message) = &session.error {
        frame.render_widget(
            Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: false }),
            error,
        );
    }
    frame.render_widget(
        Paragraph::new("Enter - start | u - all available | Ctrl+A - select all | Esc - menu")
            .wrap(Wrap { trim: false }),
        footer,
    );
}

pub fn render_translate_word(context: &mut AppContext, store: &Store, frame: &mut Frame) {
    let Some(session) = &mut context.translation_session else {
        frame.render_widget(
            Paragraph::new("No active session. Esc - menu."),
            frame.area(),
        );
        return;
    };
    let Some(phrase) = session.current_phrase() else {
        frame.render_widget(
            Paragraph::new("No phrases left. Esc - results."),
            frame.area(),
        );
        return;
    };
    let answered = session.last_answer_correct.is_some();
    let (source_language, target_language) = session.direction.language_ids(phrase);
    let [header, content, input, feedback, error, footer] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(if answered { 0 } else { 5 }),
        Constraint::Length(if answered { 1 } else { 0 }),
        Constraint::Length(if session.error.is_some() { 3 } else { 0 }),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    frame.render_widget(
        Paragraph::new(format!(
            "{:?} | {} -> {} | Phrase {}/{}{}\nCorrect: {} | Incorrect: {}",
            session.mode,
            language_name(store, source_language),
            language_name(store, target_language),
            session.current_idx + 1,
            session.queue.len(),
            if session.unlimited {
                " | All available"
            } else {
                ""
            },
            session.stats.correct,
            session.stats.incorrect,
        ))
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: false }),
        header,
    );

    let text = if answered {
        format!(
            "Prompt:\n{}\n\nYour answer:\n{}\n\nCorrect translation:\n{}\n\nLevel: {:?} | Consecutive correct answers: {}/{}",
            session.direction.prompt(phrase),
            session.input.lines().join("\n"),
            session.direction.expected_answer(phrase),
            phrase.memorizing_context.current_step,
            phrase.memorizing_context.current_attempt,
            phrase.memorizing_context.needed_attempts
        )
    } else {
        session.direction.prompt(phrase).to_owned()
    };
    let block = Block::default()
        .title(format!(" Phrase #{} ", phrase.id))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let inner = block.inner(content);
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    let max_scroll = if inner.width == 0 || inner.height == 0 {
        0
    } else {
        paragraph
            .line_count(inner.width)
            .saturating_sub(usize::from(inner.height))
    };
    session.phrase_scroll = session
        .phrase_scroll
        .min(u16::try_from(max_scroll).unwrap_or(u16::MAX));
    frame.render_widget(block, content);
    frame.render_widget(paragraph.scroll((session.phrase_scroll, 0)), inner);
    if !answered {
        frame.render_widget(&session.input, input);
    }
    if let Some(correct) = session.last_answer_correct {
        let (message, color) = if correct {
            ("Correct. Progress saved.", Color::Green)
        } else {
            ("Incorrect. Progress saved.", Color::Red)
        };
        frame.render_widget(
            Paragraph::new(message).style(Style::default().fg(color)),
            feedback,
        );
    }
    if let Some(message) = &session.error {
        frame.render_widget(
            Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: false }),
            error,
        );
    }
    let hint = if answered {
        "Enter - continue | Esc - results\nPgUp/PgDn - scroll phrase and answer"
    } else {
        "Enter - check | Ctrl+J - new line | Esc - results\nPgUp/PgDn - scroll phrase"
    };
    frame.render_widget(Paragraph::new(hint).wrap(Wrap { trim: false }), footer);
}

pub fn render_translation_result(context: &mut AppContext, store: &Store, frame: &mut Frame) {
    let Some(session) = &context.translation_session else {
        frame.render_widget(
            Paragraph::new("No active session. Enter/Esc - menu."),
            frame.area(),
        );
        return;
    };
    let [header, content, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    frame.render_widget(
        Paragraph::new(format!(
            "Translation results | {:?}\n{}",
            session.mode,
            language_pair(store)
        ))
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: false }),
        header,
    );
    let stats = &session.stats;
    let answered = stats.correct as usize + stats.incorrect as usize;
    let status = if session.queue.is_empty() {
        "No phrases are due for this mode and language pair."
    } else if answered < session.queue.len() {
        "Session stopped. Completed answers have been saved."
    } else {
        "Session complete."
    };
    frame.render_widget(Paragraph::new(format!(
        "{status}\n\nAnswered: {answered}/{}\nCorrect: {}\nIncorrect: {}\nLeveled up: {}\nLeveled down: {}\nSame level: {}",
        session.queue.len(), stats.correct, stats.incorrect, stats.leveled_up, stats.leveled_down, stats.same_level,
    )).wrap(Wrap { trim: false }), content);
    frame.render_widget(Paragraph::new("Enter/Esc - translation menu"), footer);
}

fn language_pair(store: &Store) -> String {
    format!(
        "{} / {}",
        language_name(store, store.settings.default_original_language_id),
        language_name(store, store.settings.default_translation_language_id)
    )
}

fn language_name(store: &Store, id: u32) -> &str {
    store
        .languages
        .iter()
        .find(|language| language.id == id)
        .map(|language| language.name.as_str())
        .unwrap_or("Unknown language")
}
