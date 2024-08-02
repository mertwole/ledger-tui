use std::time::Instant;

use copypasta::{ClipboardContext, ClipboardProvider};
use ratatui::crossterm::event::{Event, KeyCode};

use super::Model;
use crate::screen::{EventExt, OutgoingMessage};

pub(super) fn process_input(event: &Event, model: &mut Model) -> Option<OutgoingMessage> {
    if event.is_key_pressed(KeyCode::Char('q')) {
        return Some(OutgoingMessage::Exit);
    }

    if event.is_key_pressed(KeyCode::Char('b')) {
        return Some(OutgoingMessage::Back);
    }

    if event.is_key_pressed(KeyCode::Char('c')) {
        model.last_address_copy = Some(Instant::now());

        let state = model
            .state
            .as_ref()
            .expect("Construct should be called at the start of window lifetime");

        let pubkey = state
            .selected_account
            .as_ref()
            .expect("Selected account should be present in state") // TODO: Enforce this rule at `app` level?
            .1
            .get_info()
            .pk;

        let mut ctx = ClipboardContext::new().unwrap();
        ctx.set_contents(pubkey).unwrap();
        // It's a bug in `copypasta`. Without calling `get_contents` after `set_contents` clipboard will contain nothing.
        ctx.get_contents().unwrap();
    }

    None
}
