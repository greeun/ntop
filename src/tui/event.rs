// TUI event handling

use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent};
use tokio::sync::mpsc;

/// Events that the TUI event loop can produce.
#[derive(Debug)]
pub enum AppEvent {
    /// A key was pressed.
    Key(KeyEvent),
    /// A periodic tick for refreshing data.
    Tick,
    /// The terminal was resized.
    Resize(u16, u16),
}

/// Spawns a background task that polls crossterm events and sends them
/// through an mpsc channel.
pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
    // Keep the sender handle alive so the background task doesn't get dropped
    _tx: mpsc::UnboundedSender<AppEvent>,
}

impl EventHandler {
    /// Create a new event handler. `tick_rate` controls how often Tick events
    /// are generated when no input events occur.
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let sender = tx.clone();

        // Spawn a blocking thread (not a tokio task) since crossterm::event::poll
        // blocks the thread.
        std::thread::spawn(move || {
            loop {
                // Poll for crossterm events with the tick rate as timeout
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(Event::Key(key)) => {
                            // Only send key press events, not release/repeat
                            if key.kind == crossterm::event::KeyEventKind::Press {
                                if sender.send(AppEvent::Key(key)).is_err() {
                                    break;
                                }
                            }
                        }
                        Ok(Event::Resize(w, h)) => {
                            if sender.send(AppEvent::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                } else {
                    // No event within tick_rate — emit a Tick
                    if sender.send(AppEvent::Tick).is_err() {
                        break;
                    }
                }
            }
        });

        Self { rx, _tx: tx }
    }

    /// Receive the next event. Returns `None` if the channel is closed.
    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
