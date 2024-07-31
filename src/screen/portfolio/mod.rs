use std::collections::HashMap;

use futures::executor::block_on;
use ratatui::{crossterm::event::Event, Frame};
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
    ledger_api: L,
    coin_price_api: C,

    selected_account: Option<(NetworkIdx, AccountIdx)>,
    // TODO: Store it in API cache.
    coin_prices: HashMap<Network, Option<Decimal>>,

    state: Option<StateRegistry>,
}

type AccountIdx = usize;
type NetworkIdx = usize;

impl<L: LedgerApiT, C: CoinPriceApiT> Model<L, C> {
    pub fn new(ledger_api: L, coin_price_api: C) -> Self {
        Self {
            ledger_api,
            coin_price_api,
            selected_account: None,
            coin_prices: Default::default(),
            state: None,
        }
    }

    fn tick_logic(&mut self) {
        let state = self
            .state
            .as_mut()
            .expect("Construct should be called at the start of window lifetime");

        if state.device_accounts.is_none() {
            if let Some((active_device, _)) = state.active_device.as_ref() {
                // TODO: Load at startup from config and add only on user request.
                // TODO: Filter accounts based on balance.
                state.device_accounts = Some(
                    [Network::Bitcoin, Network::Ethereum]
                        .into_iter()
                        .filter_map(|network| {
                            let accounts: Vec<_> =
                                block_on(self.ledger_api.discover_accounts(active_device, network))
                                    .collect();

                            if accounts.is_empty() {
                                None
                            } else {
                                Some((network, accounts))
                            }
                        })
                        .collect(),
                );
            }
        }

        // TODO: Correctly map accounts to coins.
        // TODO: Don't request price each tick.
        self.coin_prices = [Network::Bitcoin, Network::Ethereum]
            .into_iter()
            .map(|network| {
                let coin = match network {
                    Network::Bitcoin => Coin::BTC,
                    Network::Ethereum => Coin::ETH,
                };

                (
                    network,
                    block_on(self.coin_price_api.get_price(coin, Coin::USDT)),
                )
            })
            .collect();
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT> Screen for Model<L, C> {
    fn construct(&mut self, state: StateRegistry) {
        self.state = Some(state);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        view::render(self, frame);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic();

        controller::process_input(self, event.as_ref()?)
    }

    fn deconstruct(self: Box<Self>) -> StateRegistry {
        self.state
            .expect("Construct should be called at the start of window lifetime")
    }
}
