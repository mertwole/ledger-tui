use std::time::Duration;

use ratatui::crossterm::event::{self, KeyCode};

use crate::window::{EventExt, OutgoingMessage};

pub(super) fn process_input() -> Option<OutgoingMessage> {
    if !event::poll(Duration::ZERO).unwrap() {
        return None;
    }

    let event = event::read().unwrap();

    if event.is_key_pressed(KeyCode::Char('q')) {
        return Some(OutgoingMessage::Exit);
    }

    None
}
