use std::collections::HashMap;

use futures::executor::block_on;
use ratatui::{crossterm::event::Event, Frame};
use rust_decimal::Decimal;

use super::{OutgoingMessage, ScreenT};
use crate::{
    api::{
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
    selected_account: Option<(NetworkIdx, AccountIdx)>,
    // TODO: Store it in API cache.
    coin_prices: HashMap<Network, Option<Decimal>>,
    balances: HashMap<(Network, Account), Decimal>,

    state: Option<StateRegistry>,
    apis: ApiRegistry<L, C, M>,
}

type AccountIdx = usize;
type NetworkIdx = usize;

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    pub fn new(api_registry: ApiRegistry<L, C, M>) -> Self {
        Self {
            selected_account: None,
            coin_prices: Default::default(),
            balances: Default::default(),

            state: None,
            apis: api_registry,
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
                            let accounts = block_on(
                                self.apis
                                    .ledger_api
                                    .discover_accounts(active_device, network),
                            );

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
                    block_on(self.apis.coin_price_api.get_price(coin, Coin::USDT)),
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
                                self.apis
                                    .blockchain_monitoring_api
                                    .get_balance(*network, account),
                            )
                        });
                }
            }
        }
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, api_registry: ApiRegistry<L, C, M>) -> Self {
        Self {
            selected_account: None,
            coin_prices: Default::default(),
            balances: Default::default(),

            state: Some(state),
            apis: api_registry,
        }
    }

    fn render(&self, frame: &mut Frame<'_>) {
        view::render(self, frame);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic();

        controller::process_input(self, event.as_ref()?)
    }

    fn deconstruct(self) -> (StateRegistry, ApiRegistry<L, C, M>) {
        (self.state.unwrap(), self.apis)
    }
}
