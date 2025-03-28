use copypasta::{ClipboardContext, ClipboardProvider};
use input_mapping_common::InputMappingT;
use input_mapping_derive::InputMapping;
use ratatui::crossterm::event::Event;

use super::Model;
use crate::{api::ledger::LedgerApiT, screen::OutgoingMessage};

#[derive(InputMapping)]
pub enum InputEvent {
    #[key = 'q']
    #[description = "Quit application"]
    Quit,

    #[key = 'h']
    #[description = "Open/close navigation help"]
    NavigationHelp,

    #[key = 'b']
    #[description = "Return one screen back"]
    Back,

    #[key = 'p']
    #[description = "Paste a receiver address"]
    PasteAddress,
}

pub(super) fn process_input<L: LedgerApiT>(
    event: &Event,
    model: &mut Model<L>,
) -> Option<OutgoingMessage> {
    let event = InputEvent::map_event(event.clone())?;

    match event {
        InputEvent::Quit => Some(OutgoingMessage::Exit),
        InputEvent::NavigationHelp => {
            model.show_navigation_help ^= true;
            None
        }
        InputEvent::Back => Some(OutgoingMessage::Back),
        InputEvent::PasteAddress => {
            let mut ctx = ClipboardContext::new().unwrap();
            let address = ctx.get_contents().unwrap();

            model.receiver_address = Some(address);

            None
        }
    }
}
