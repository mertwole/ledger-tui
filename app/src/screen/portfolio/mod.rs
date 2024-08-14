use std::collections::HashMap;

use futures::executor::block_on;
use ratatui::{crossterm::event::Event, Frame};
use rust_decimal::Decimal;

use super::{OutgoingMessage, Screen};
use crate::{
    api::{
        self,
        blockchain_monitoring::BlockchainMonitoringApiT,
        coin_price::{Coin, CoinPriceApiT},
        common::{Account, Network},
        ledger::LedgerApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    ledger_api: L,
    coin_price_api: C,
    blockchain_monitoring_api: M,

    selected_account: Option<(NetworkIdx, AccountIdx)>,
    // TODO: Store it in API cache.
    coin_prices: HashMap<Network, Option<Decimal>>,
    balances: HashMap<(Network, Account), Decimal>,

    state: Option<StateRegistry>,
}

type AccountIdx = usize;
type NetworkIdx = usize;

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    pub fn new(api_registry: ApiRegistry<L, C, M>) -> Self {
        Self {
            ledger_api: api_registry.ledger_api,
            coin_price_api: api_registry.coin_price_api,
            blockchain_monitoring_api: api_registry.blockchain_monitoring_api,

            selected_account: None,
            coin_prices: Default::default(),
            balances: Default::default(),
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
                            let accounts =
                                block_on(self.ledger_api.discover_accounts(active_device, network));

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

        // TODO: Don't request balance each tick.
        if let Some(accounts) = state.device_accounts.as_ref() {
            for (network, accounts) in accounts {
                for account in accounts {
                    self.balances
                        .entry((*network, account.clone()))
                        .or_insert_with(|| {
                            block_on(
                                self.blockchain_monitoring_api
                                    .get_balance(*network, account),
                            )
                        });
                }
            }
        }
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Screen for Model<L, C, M> {
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
