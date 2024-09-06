use std::time::Instant;

use copypasta::{ClipboardContext, ClipboardProvider};
use input_mapping_common::InputMappingT;
use input_mapping_derive::InputMapping;
use ratatui::crossterm::event::Event;

use super::Model;
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        ledger::LedgerApiT,
    },
    screen::OutgoingMessage,
};

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

    #[key = 'c']
    #[description = "Copy address to a clipboard"]
    CopyAddress,
}

pub(super) fn process_input<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    event: &Event,
    model: &mut Model<L, C, M>,
) -> Option<OutgoingMessage> {
    let event = InputEvent::map_event(event.clone())?;

    match event {
        InputEvent::Quit => Some(OutgoingMessage::Exit),
        InputEvent::NavigationHelp => {
            model.show_navigation_help ^= true;
            None
        }
        InputEvent::Back => Some(OutgoingMessage::Back),
        InputEvent::CopyAddress => {
            model.last_address_copy = Some(Instant::now());

            let pubkey = model
                .state
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

            None
        }
    }
}
