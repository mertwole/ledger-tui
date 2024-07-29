use std::time::Duration;

use ratatui::crossterm::event::{self, KeyCode};

use crate::screen::{EventExt, OutgoingMessage, ScreenName};

pub(super) fn process_input() -> Option<OutgoingMessage> {
    if !event::poll(Duration::ZERO).unwrap() {
        return None;
    }

    let event = event::read().unwrap();

    if event.is_key_pressed(KeyCode::Char('q')) {
        return Some(OutgoingMessage::Exit);
    }

    if event.is_key_pressed(KeyCode::Char('b')) {
        return Some(OutgoingMessage::Back);
    }

    if event.is_key_pressed(KeyCode::Char('d')) {
        return Some(OutgoingMessage::SwitchScreen(ScreenName::Deposit));
    }

    None
}
