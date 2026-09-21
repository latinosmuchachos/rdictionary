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
    sender: mpsc::Sender<KeyEvent>,
    receiver: mpsc::Receiver<KeyEvent>,
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
                        match crossterm_event::read().expect("unable to read event") {
                            CrosstermEvent::Key(e) => {
                                if e.kind == crossterm_event::KeyEventKind::Press {
                                    sender.send(e)
                                } else {
                                    Ok(())
                                }
                            }
                            _ => Ok(()),
                        }
                        .expect("failed to send terminal event")
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

    pub fn next(&self) -> Result<KeyEvent> {
        Ok(self.receiver.recv()?)
    }
}
