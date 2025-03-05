use std::time::Instant;

use ratatui::{Frame, crossterm::event::Event};

use super::{OutgoingMessage, ScreenT, resources::Resources};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        ledger::LedgerApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

pub struct Model {
    last_address_copy: Option<Instant>,
    show_navigation_help: bool,

    state: StateRegistry,
}

impl Model {
    pub fn construct<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
        state: StateRegistry,
        api_registry: ApiRegistry<L, C, M>,
    ) -> (Self, ApiRegistry<L, C, M>) {
        (
            Self {
                last_address_copy: None,
                show_navigation_help: false,

                state,
            },
            api_registry,
        )
    }

    pub async fn deconstruct<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
        self,
        api_registry: ApiRegistry<L, C, M>,
    ) -> (StateRegistry, ApiRegistry<L, C, M>) {
        (self.state, api_registry)
    }
}

impl ScreenT for Model {
    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        controller::process_input(event.as_ref()?, self)
    }
}
