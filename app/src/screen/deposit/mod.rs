use std::time::Instant;

use ratatui::{crossterm::event::Event, Frame};

use super::{OutgoingMessage, Screen};
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

    state: Option<StateRegistry>,
}

impl Model {
    pub fn new<L, C, M>(_api_registry: ApiRegistry<L, C, M>) -> Self
    where
        L: LedgerApiT,
        C: CoinPriceApiT,
        M: BlockchainMonitoringApiT,
    {
        Self {
            last_address_copy: None,
            state: None,
        }
    }
}

impl Screen for Model {
    fn construct(&mut self, state: StateRegistry) {
        self.state = Some(state);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        view::render(self, frame);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        controller::process_input(event.as_ref()?, self)
    }

    fn deconstruct(self: Box<Self>) -> StateRegistry {
        self.state
            .expect("Construct should be called at the start of window lifetime")
    }
}
