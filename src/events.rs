use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use color_eyre::Result;
use ratatui::crossterm::event::{self as crossterm_event, Event as CrosstermEvent, KeyEvent};

#[derive(Debug)]
pub struct EventHandler {
    #[allow(dead_code)]
    sender: mpsc::Sender<CrosstermEvent>,
    receiver: mpsc::Receiver<CrosstermEvent>,
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new() -> Self {
        let tick_rate = Duration::from_millis(100);

        let (sender, receiver) = mpsc::channel();

        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if crossterm_event::poll(timeout).expect("unable to poll for event") {
                        let event = crossterm_event::read().expect("unable to read event");
                        if matches!(
                            event,
                            CrosstermEvent::Key(KeyEvent {
                                kind: crossterm_event::KeyEventKind::Press,
                                ..
                            }) | CrosstermEvent::Resize(_, _)
                        ) && sender.send(event).is_err()
                        {
                            break;
                        }
                    }

                    if last_tick.elapsed() >= tick_rate {
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub fn next(&self) -> Result<CrosstermEvent> {
        Ok(self.receiver.recv()?)
    }
}
