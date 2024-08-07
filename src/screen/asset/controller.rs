use ratatui::crossterm::event::{Event, KeyCode};

use crate::screen::{EventExt, OutgoingMessage, ScreenName};

pub(super) fn process_input(event: &Event) -> Option<OutgoingMessage> {
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

fn process_time_interval_selection(event: &Event) {
    match () {
        () if event.is_key_pressed(KeyCode::Char('d')) => {
            // TODO
        }
        () if event.is_key_pressed(KeyCode::Char('w')) => {
            // TODO
        }
        () if event.is_key_pressed(KeyCode::Char('m')) => {
            // TODO
        }
        () if event.is_key_pressed(KeyCode::Char('y')) => {
            // TODO
        }
        () if event.is_key_pressed(KeyCode::Char('a')) => {
            // TODO
        }
        _ => {}
    };
}
