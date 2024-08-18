use std::time::Instant;

use ratatui::{crossterm::event::Event, Frame};

use super::{OutgoingMessage, ScreenT};
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

    state: Option<StateRegistry>,
    apis: ApiRegistry<L, C, M>,
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    pub fn new(api_registry: ApiRegistry<L, C, M>) -> Self {
        Self {
            last_address_copy: None,

            state: None,
            apis: api_registry,
        }
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, api_registry: ApiRegistry<L, C, M>) -> Self {
        Self {
            last_address_copy: None,

            state: Some(state),
            apis: api_registry,
        }
    }

    fn render(&self, frame: &mut Frame<'_>) {
        view::render(self, frame);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        controller::process_input(event.as_ref()?, self)
    }

    fn deconstruct(self) -> (StateRegistry, ApiRegistry<L, C, M>) {
        (self.state.unwrap(), self.apis)
    }
}
