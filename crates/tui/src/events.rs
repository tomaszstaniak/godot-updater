use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

/// Poll for events with a timeout.
/// Only returns KeyPress events — ignores KeyRelease/KeyRepeat to avoid double-firing.
pub fn poll_event(timeout: Duration) -> Option<AppEvent> {
    if event::poll(timeout).ok()? {
        match event::read().ok()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => Some(AppEvent::Key(key)),
            _ => Some(AppEvent::Tick),
        }
    } else {
        Some(AppEvent::Tick)
    }
}

/// Check if key event is quit (Q or Ctrl+C).
pub fn is_quit(key: &KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } | KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    )
}
