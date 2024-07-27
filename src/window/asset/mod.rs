use ratatui::Frame;

use super::{OutgoingMessage, Window};
use crate::{
    api::{coin_price::CoinPriceApiT, ledger::LedgerApiT},
    app::StateRegistry,
};

mod controller;
mod view;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT> {
    _ledger_api: L,
    coin_price_api: C,

    state: Option<StateRegistry>,
}

impl<L: LedgerApiT, C: CoinPriceApiT> Model<L, C> {
    pub fn new(ledger_api: L, coin_price_api: C) -> Self {
        Self {
            _ledger_api: ledger_api,
            coin_price_api,
            state: None,
        }
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT> Window for Model<L, C> {
    fn construct(&mut self, state: StateRegistry) {
        self.state = Some(state);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        view::render(&self, frame);
    }

    fn tick(&mut self) -> Option<OutgoingMessage> {
        controller::process_input()
    }

    fn deconstruct(self: Box<Self>) -> StateRegistry {
        self.state
            .expect("Construct should be called at the start of window lifetime")
    }
}
