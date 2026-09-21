mod app;

mod events;

mod menu;

mod models;

mod storage;

mod tui;

mod ui;

use app::App;
use color_eyre::Result;
use tui::Tui;

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.start()?;

    while !app.context.should_quit {
        Tui::tick(&mut app)?;
    }

    app.exit()?;
    Ok(())
}
