use std::{marker::PhantomData, sync::Arc, time::Instant};

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

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    last_address_copy: Option<Instant>,
    show_navigation_help: bool,

    state: StateRegistry,
    _phantom: PhantomData<(L, C, M)>,
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, _api_registry: Arc<ApiRegistry<L, C, M>>) -> Self {
        Self {
            last_address_copy: None,
            show_navigation_help: false,

            state,
            _phantom: PhantomData,
        }
    }

    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        controller::process_input(event.as_ref()?, self)
    }

    fn deconstruct(self) -> StateRegistry {
        self.state
    }
}
