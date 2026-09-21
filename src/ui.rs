use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::AppContext;

pub fn render_main_menu(_app_context: &mut AppContext, frame: &mut Frame) {
    let msg = "Let's start translate the words!\n\
    Choose the appropriate action:\n\
    t - translate words\n\
    e - edit your dictionary\n\
    q - quit from app";

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
}

pub fn render_translate_menu(_app_context: &mut AppContext, frame: &mut Frame) {
    let msg = "Choose option:\n\
    d - daily words\n\
    w - weekly words\n\
    m - monthly words\n\
    Esc - go back\n\
    q - quit from app";

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
}

pub fn render_how_many_will_translate(app_context: &mut AppContext, frame: &mut Frame) {
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
        Constraint::Length(2), // вопрос
        Constraint::Length(3), // поле (1 строка + 2 рамки)
        Constraint::Min(0),    // подсказка/ошибка
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

pub fn render_translate_word(app_context: &mut AppContext, frame: &mut Frame) {
    if let Some(translation_context) = app_context.translation_context {
        let msg = format!(
            "You should translate {} {:?} words!\n\
            (Esc - go back, q - quit from app)",
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
