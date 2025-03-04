use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bigdecimal::BigDecimal;
use futures::{executor::block_on, future::join_all};
use itertools::Itertools;
use ratatui::{Frame, crossterm::event::Event};
use rust_decimal::Decimal;
use tokio::task::JoinHandle;

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

type TaskHandle<R> = Option<JoinHandle<R>>;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    selected_account: Option<(NetworkIdx, AccountIdx)>,
    // TODO: Store it in API cache.
    coin_prices: Arc<Mutex<HashMap<Network, Option<Decimal>>>>,
    balances: Arc<Mutex<HashMap<(Network, Account), BigDecimal>>>,
    show_navigation_help: bool,

    state: StateRegistry,
    apis: Arc<ApiRegistry<L, C, M>>,

    coin_price_task: TaskHandle<HashMap<Network, Option<Decimal>>>,
}

type AccountIdx = usize;
type NetworkIdx = usize;

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    async fn init_logic(&mut self) {
        let active_device = self
            .state
            .active_device
            .clone()
            .expect("TODO: Enforce this rule at app level?")
            .0;

        // TODO: Introduce separate screen where account loading and management will be performed.
        let mut device_accounts = vec![];
        for network in [Network::Bitcoin, Network::Ethereum] {
            let accounts = self
                .apis
                .ledger_api
                .discover_accounts(&active_device, network)
                .await;

            if !accounts.is_empty() {
                device_accounts.push((network, accounts));
            }
        }

        self.state.device_accounts = Some(device_accounts);
    }

    async fn tick_logic(&mut self) {
        let apis = self.apis.clone();
        let spawn_coin_price_task = || {
            tokio::task::spawn(async move {
                let prices = [Coin::BTC, Coin::ETH]
                    .map(|coin| apis.coin_price_api.get_price(coin, Coin::USDT));
                let prices = join_all(prices).await;
                let networks = [Network::Bitcoin, Network::Ethereum];

                networks.into_iter().zip_eq(prices.into_iter()).collect()
            })
        };

        self.coin_price_task = Some(match self.coin_price_task.take() {
            Some(join_handle) => {
                if join_handle.is_finished() {
                    *self.coin_prices.lock().unwrap() = join_handle.await.unwrap();

                    spawn_coin_price_task()
                } else {
                    join_handle
                }
            }
            None => spawn_coin_price_task(),
        });

        // TODO: Don't request balance each tick.
        for (network, accounts) in self
            .state
            .device_accounts
            .as_ref()
            .expect("TODO: Enforce this rule at app level?")
        {
            for account in accounts {
                if !self
                    .balances
                    .lock()
                    .expect("Failed to acquire lock on mutex")
                    .contains_key(&(*network, account.clone()))
                {
                    let apis = self.apis.clone();
                    let balances = self.balances.clone();

                    let account = account.clone();
                    let network = network.clone();

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

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, api_registry: Arc<ApiRegistry<L, C, M>>) -> Self {
        let mut new = Self {
            selected_account: None,
            coin_prices: Default::default(),
            balances: Default::default(),
            show_navigation_help: false,

            state,
            apis: api_registry,

            coin_price_task: None,
        };

        block_on(new.init_logic());

        new
    }

    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic().await;

        controller::process_input(event.as_ref()?, self)
    }

    fn deconstruct(self) -> StateRegistry {
        self.state
    }
}
