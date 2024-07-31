use ratatui::crossterm::event::{Event, KeyCode};

use crate::screen::{EventExt, OutgoingMessage};

pub(super) fn process_input(event: &Event) -> Option<OutgoingMessage> {
    if event.is_key_pressed(KeyCode::Char('q')) {
        return Some(OutgoingMessage::Exit);
    }

    if event.is_key_pressed(KeyCode::Char('b')) {
        return Some(OutgoingMessage::Back);
    }

    None
}
