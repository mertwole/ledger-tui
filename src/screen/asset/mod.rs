use std::time::Instant;

use futures::executor::block_on;
use ratatui::Frame;
use rust_decimal::Decimal;

use super::{OutgoingMessage, Screen};
use crate::{
    api::{
        coin_price::{Coin, CoinPriceApiT},
        ledger::{LedgerApiT, Network},
    },
    app::StateRegistry,
};

mod controller;
mod view;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT> {
    _ledger_api: L, // TODO: Remove it.
    coin_price_api: C,

    coin_price_history: Option<Vec<(Instant, Decimal)>>,

    state: Option<StateRegistry>,
}

impl<L: LedgerApiT, C: CoinPriceApiT> Model<L, C> {
    pub fn new(ledger_api: L, coin_price_api: C) -> Self {
        Self {
            _ledger_api: ledger_api,
            coin_price_api,
            coin_price_history: None,
            state: None,
        }
    }

    fn tick_logic(&mut self) {
        let state = self
            .state
            .as_ref()
            .expect("Construct should be called at the start of window lifetime");

        let selected_account = state
            .selected_account
            .as_ref()
            .expect("Selected account should be present in state"); // TODO: Enforce this rule at `app` level?

        let coin = match selected_account.0 {
            Network::Bitcoin => Coin::BTC,
            Network::Ethereum => Coin::ETH,
        };

        // TODO: Don't make requests to API each tick.
        let mut history =
            block_on(self.coin_price_api.get_price_history(coin, Coin::USDT)).unwrap();
        history.sort_by(|a, b| a.0.cmp(&b.0));

        self.coin_price_history = Some(history);
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT> Screen for Model<L, C> {
    fn construct(&mut self, state: StateRegistry) {
        self.state = Some(state);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        view::render(self, frame);
    }

    fn tick(&mut self) -> Option<OutgoingMessage> {
        self.tick_logic();

        controller::process_input()
    }

    fn deconstruct(self: Box<Self>) -> StateRegistry {
        self.state
            .expect("Construct should be called at the start of window lifetime")
    }
}
