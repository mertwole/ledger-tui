use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bigdecimal::BigDecimal;
use futures::executor::block_on;
use ratatui::{Frame, crossterm::event::Event};
use rust_decimal::Decimal;

use super::{OutgoingMessage, ScreenT, resources::Resources};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT,
        coin_price::{Coin, CoinPriceApiT},
        common_types::{Account, Network},
        ledger::LedgerApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    selected_account: Option<(NetworkIdx, AccountIdx)>,
    // TODO: Store it in API cache.
    coin_prices: Arc<Mutex<HashMap<Network, Option<Decimal>>>>,
    balances: Arc<Mutex<HashMap<(Network, Account), BigDecimal>>>,
    show_navigation_help: bool,

    state: StateRegistry,
    apis: Arc<ApiRegistry<L, C, M>>,
}

type AccountIdx = usize;
type NetworkIdx = usize;

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    fn init_logic(&self) {
        let state_device_accounts = self.state.device_accounts.clone();
        let apis = self.apis.clone();

        if let Some((active_device, _)) = self.state.active_device.clone() {
            // TODO: Introduce separate screen where account loading and management will be performed.
            tokio::task::spawn(async move {
                let mut device_accounts = vec![];
                for network in [Network::Bitcoin, Network::Ethereum] {
                    let accounts = apis
                        .ledger_api
                        .discover_accounts(&active_device, network)
                        .await;

                    if !accounts.is_empty() {
                        device_accounts.push((network, accounts));
                    }
                }

                *state_device_accounts
                    .lock()
                    .expect("Failed to acquire lock on mutex") = Some(device_accounts);
            });
        }
    }

    async fn tick_logic(&mut self) {
        let apis = self.apis.clone();
        let state_coin_prices = self.coin_prices.clone();

        tokio::task::spawn(async move {
            let mut coin_prices = HashMap::new();
            // TODO: Correctly map accounts to coins.
            let networks = [Network::Bitcoin, Network::Ethereum];
            for network in networks {
                let coin = match network {
                    Network::Bitcoin => Coin::BTC,
                    Network::Ethereum => Coin::ETH,
                };

                let price = apis.coin_price_api.get_price(coin, Coin::USDT).await;
                coin_prices.insert(network, price);
            }

            let mut guard = state_coin_prices
                .lock()
                .expect("Failed to acquire lock on mutex");
            *guard = coin_prices;
        });

        // TODO: Don't request balance each tick.
        let device_accounts = self
            .state
            .device_accounts
            .lock()
            .expect("Failed to acquire lock on mutex");
        if let Some(accounts) = device_accounts.clone() {
            for (network, accounts) in accounts {
                for account in accounts {
                    if !self
                        .balances
                        .lock()
                        .expect("Failed to acquire lock on mutex")
                        .contains_key(&(network, account.clone()))
                    {
                        let apis = self.apis.clone();
                        let balances = self.balances.clone();

                        tokio::task::spawn(async move {
                            let balance = apis
                                .blockchain_monitoring_api
                                .get_balance(network, &account)
                                .await;

                            balances
                                .lock()
                                .expect("Failed to acquire lock on mutex")
                                .insert((network, account), balance);
                        });
                    }
                }
            }
        }
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, api_registry: Arc<ApiRegistry<L, C, M>>) -> Self {
        let new = Self {
            selected_account: None,
            coin_prices: Default::default(),
            balances: Default::default(),
            show_navigation_help: false,

            state,
            apis: api_registry,
        };

        new.init_logic();

        new
    }

    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        block_on(self.tick_logic());

        controller::process_input(event.as_ref()?, self)
    }

    fn deconstruct(self) -> StateRegistry {
        self.state
    }
}
