/// Converts crossterm events into application events.
use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

/// Application-level events produced by the event handler.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A key was pressed.
    Key(KeyEvent),
    /// An animation tick (~33ms).
    Tick,
    /// The terminal was resized.
    Resize(u16, u16),
}

/// Polls crossterm and produces [`AppEvent`]s.
pub struct EventHandler;

impl EventHandler {
    /// Polls for the next event.
    ///
    /// When `animating` is true, uses a 33ms timeout so tick events are
    /// generated at ~30 fps. Otherwise blocks indefinitely until user input.
    pub fn next(animating: bool) -> std::io::Result<AppEvent> {
        let timeout = if animating {
            Duration::from_millis(33)
        } else {
            Duration::from_secs(u64::MAX / 2)
        };

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => Ok(AppEvent::Key(key)),
                Event::Resize(w, h) => Ok(AppEvent::Resize(w, h)),
                // Ignore key release/repeat and other events — emit tick
                _ => Ok(AppEvent::Tick),
            }
        } else {
            Ok(AppEvent::Tick)
        }
    }
}
