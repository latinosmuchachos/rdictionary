use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

mod add_phrase;
mod edit_phrase;
mod edit_phrase_confirm;
mod phrase_browser;
mod settings;
mod translation;
pub use add_phrase::render_add_phrase;
pub use edit_phrase::render_edit_phrase;
pub use edit_phrase_confirm::render_edit_phrase_confirm;
pub use phrase_browser::render_phrase_browser;
pub use settings::{render_settings_attempts, render_settings_languages, render_settings_menu};
pub use translation::{
    render_how_many_will_translate, render_translate_word, render_translation_result,
};

use crate::{
    app::AppContext,
    menu::{EDIT_DICTIONARY_MENU_ITEMS, MAIN_MENU_ITEMS, TRANSLATE_MENU_ITEMS},
    storage::Store,
};

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
