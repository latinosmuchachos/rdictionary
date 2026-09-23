mod app;

mod events;

mod menu;

mod models;

mod storage;

mod tui;

mod ui;

use app::App;
use color_eyre::Result;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{Layer, fmt::layer, layer::SubscriberExt, util::SubscriberInitExt};
use tui::Tui;

fn main() -> Result<()> {
    let _log_guard = log_settings();
    tracing::info!("Starting rdictionary application!");

    let mut app = App::new()?;
    app.start()?;

    while !app.context.should_quit {
        Tui::tick(&mut app)?;
    }

    app.exit()?;
    tracing::info!("Exit from rdictionary application!");
    Ok(())
}

fn log_settings() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::daily("logs", "rdictionary.log");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("rdictionary=debug".parse().unwrap());

    let file_layer = layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false)
        .with_line_number(true)
        .with_filter(file_filter);

    tracing_subscriber::registry().with(file_layer).init();

    guard
}
