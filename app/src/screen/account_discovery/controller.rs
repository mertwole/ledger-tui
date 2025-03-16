use input_mapping_common::InputMappingT;
use input_mapping_derive::InputMapping;
use ratatui::crossterm::event::Event;

use crate::{
    api::{common_types::Network, ledger::LedgerApiT, storage::StorageApiT},
    screen::OutgoingMessage,
};

use super::Model;

#[derive(InputMapping)]
pub enum InputEvent {
    #[key = 'd']
    #[description = "Discover Bitcoin accounts"]
    Discover,

    #[key = 'q']
    #[description = "Quit application"]
    Quit,

    #[key = 'h']
    #[description = "Open/close navigation help"]
    NavigationHelp,

    #[key = 'b']
    #[description = "Return one screen back"]
    Back,
}

pub(super) async fn process_input<L: LedgerApiT, S: StorageApiT>(
    event: &Event,
    model: &mut Model<L, S>,
) -> Option<OutgoingMessage> {
    let event = InputEvent::map_event(event.clone())?;

    match event {
        InputEvent::Discover => {
            model.fetch_accounts(Network::Bitcoin).await;
            None
        }
        InputEvent::Quit => Some(OutgoingMessage::Exit),
        InputEvent::NavigationHelp => {
            model.show_navigation_help ^= true;
            None
        }
        InputEvent::Back => Some(OutgoingMessage::Back),
    }
}
