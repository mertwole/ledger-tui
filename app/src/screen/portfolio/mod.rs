use std::collections::HashMap;

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
    coin_prices: HashMap<Network, Option<Decimal>>,
    balances: HashMap<(Network, Account), BigDecimal>,
    show_navigation_help: bool,

    state: StateRegistry,
    apis: ApiSubRegistry<L, C, M>,

    coin_price_task: TaskHandle<HashMap<Network, Option<Decimal>>>,
    account_balances_task: TaskHandle<HashMap<(Network, Account), BigDecimal>>,
}

pub struct ApiSubRegistry<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    ledger_api: Option<L>,
    coin_price_api: Option<C>,
    blockchain_monitoring_api: Option<M>,
}

type AccountIdx = usize;
type NetworkIdx = usize;

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    pub fn construct(
        state: StateRegistry,
        mut api_registry: ApiRegistry<L, C, M>,
    ) -> (Self, ApiRegistry<L, C, M>) {
        let apis = ApiSubRegistry {
            ledger_api: api_registry.ledger_api.take(),
            coin_price_api: api_registry.coin_price_api.take(),
            blockchain_monitoring_api: api_registry.blockchain_monitoring_api.take(),
        };

        let mut new = Self {
            selected_account: None,
            coin_prices: Default::default(),
            balances: Default::default(),
            show_navigation_help: false,

            state,
            apis,

            coin_price_task: None,
            account_balances_task: None,
        };

        block_on(new.init_logic());

        (new, api_registry)
    }

    async fn init_logic(&mut self) {
        let active_device = self
            .state
            .active_device
            .clone()
            .expect("TODO: Enforce this rule at app level?")
            .0;

        // TODO: Introduce separate screen where account loading and management will be performed.
        let mut device_accounts = vec![];
        // TODO: Fix.
        let ledger_api = self.apis.ledger_api.take().unwrap();
        for network in [Network::Bitcoin, Network::Ethereum] {
            let accounts = ledger_api.discover_accounts(&active_device, network).await;

            if !accounts.is_empty() {
                device_accounts.push((network, accounts));
            }
        }

        self.state.device_accounts = Some(device_accounts);
    }

    async fn tick_logic(&mut self) {
        // TODO: Fix.
        let coin_price_api = self.apis.coin_price_api.take().unwrap();
        let spawn_coin_price_task = || {
            tokio::task::spawn(async move {
                let prices =
                    [Coin::BTC, Coin::ETH].map(|coin| coin_price_api.get_price(coin, Coin::USDT));
                let prices = join_all(prices).await;
                let networks = [Network::Bitcoin, Network::Ethereum];

                networks.into_iter().zip_eq(prices.into_iter()).collect()
            })
        };

        self.coin_price_task = Some(match self.coin_price_task.take() {
            Some(join_handle) => {
                if join_handle.is_finished() {
                    self.coin_prices = join_handle.await.unwrap();

                    spawn_coin_price_task()
                } else {
                    join_handle
                }
            }
            None => spawn_coin_price_task(),
        });

        // TODO: Fix.
        let blockchain_monitoring_api = self.apis.blockchain_monitoring_api.take().unwrap();
        let accounts = self
            .state
            .device_accounts
            .clone()
            .expect("TODO: Enforce this rule at app level?");
        let spawn_account_balances_task = || {
            tokio::task::spawn(async move {
                let accounts: Vec<_> = accounts
                    .into_iter()
                    .flat_map(|(network, accounts)| {
                        accounts.into_iter().map(move |account| (network, account))
                    })
                    .collect();

                let balances = accounts.iter().map(|(network, account)| {
                    blockchain_monitoring_api.get_balance(*network, account)
                });
                let balances = join_all(balances).await;

                accounts.into_iter().zip_eq(balances).collect()
            })
        };

        // TODO: Request balances only when user updates the screen.
        self.account_balances_task = Some(match self.account_balances_task.take() {
            Some(join_handle) => {
                if join_handle.is_finished() {
                    self.balances = join_handle.await.unwrap();

                    spawn_account_balances_task()
                } else {
                    join_handle
                }
            }
            None => spawn_account_balances_task(),
        });
    }

    pub async fn deconstruct(
        mut self,
        mut api_registry: ApiRegistry<L, C, M>,
    ) -> (StateRegistry, ApiRegistry<L, C, M>) {
        api_registry.ledger_api = self.apis.ledger_api.take();
        api_registry.coin_price_api = self.apis.coin_price_api.take();
        api_registry.blockchain_monitoring_api = self.apis.blockchain_monitoring_api.take();

        (self.state, api_registry)
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT for Model<L, C, M> {
    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic().await;

        controller::process_input(event.as_ref()?, self)
    }
}
