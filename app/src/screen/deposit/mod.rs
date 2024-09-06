use std::time::Instant;

use ratatui::{crossterm::event::Event, Frame};

use super::{resources::Resources, OutgoingMessage, ScreenT};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        ledger::LedgerApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    last_address_copy: Option<Instant>,
    show_navigation_help: bool,

    state: StateRegistry,
    apis: ApiRegistry<L, C, M>,
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, api_registry: ApiRegistry<L, C, M>) -> Self {
        Self {
            last_address_copy: None,
            show_navigation_help: false,

            state,
            apis: api_registry,
        }
    }

    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        controller::process_input(event.as_ref()?, self)
    }

    fn deconstruct(self) -> (StateRegistry, ApiRegistry<L, C, M>) {
        (self.state, self.apis)
    }
}
