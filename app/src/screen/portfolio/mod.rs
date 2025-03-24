use std::collections::HashMap;

use bigdecimal::BigDecimal;
use futures::future::join_all;
use itertools::Itertools;
use ratatui::{Frame, crossterm::event::Event};
use rust_decimal::Decimal;

use super::{OutgoingMessage, ScreenT, common::api_task::ApiTask, resources::Resources};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT,
        coin_price::{Coin, CoinPriceApiT},
        common_types::{Account, Network},
        ledger::LedgerApiT,
        storage::StorageApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

pub struct Model<C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    selected_network: Option<NetworkIdx>,
    selected_account: Option<AccountIdx>,
    coin_prices: HashMap<Network, Option<Decimal>>,
    balances: HashMap<(Network, Account), BigDecimal>,
    show_navigation_help: bool,

    state: StateRegistry,

    coin_price_task: ApiTask<C, HashMap<Network, Option<Decimal>>>,
    account_balances_task: ApiTask<M, HashMap<(Network, Account), BigDecimal>>,
}

type AccountIdx = usize;
type NetworkIdx = usize;

impl<C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<C, M> {
    pub fn construct<L: LedgerApiT, S: StorageApiT>(
        state: StateRegistry,
        mut api_registry: ApiRegistry<L, C, M, S>,
    ) -> (Self, ApiRegistry<L, C, M, S>) {
        let coin_price_task = ApiTask::new(api_registry.coin_price_api.take().unwrap());
        let account_balances_task =
            ApiTask::new(api_registry.blockchain_monitoring_api.take().unwrap());

        (
            Self {
                selected_network: None,
                selected_account: None,
                coin_prices: HashMap::new(),
                balances: HashMap::new(),
                show_navigation_help: false,

                state,

                coin_price_task,
                account_balances_task,
            },
            api_registry,
        )
    }

    async fn tick_logic(&mut self) {
        let spawn_coin_price_task = |coin_price_api: C| {
            tokio::task::spawn(async move {
                let prices =
                    [Coin::BTC, Coin::ETH].map(|coin| coin_price_api.get_price(coin, Coin::USDT));
                let prices = join_all(prices).await;
                let networks = [Network::Bitcoin, Network::Ethereum];

                let coin_prices = networks.into_iter().zip_eq(prices.into_iter()).collect();

                (coin_price_api, coin_prices)
            })
        };

        if let Some(coin_prices) = self
            .coin_price_task
            .try_fetch_value_and_rerun(spawn_coin_price_task)
            .await
        {
            self.coin_prices = coin_prices;
        }

        let accounts = self.state.device_accounts.clone();
        let spawn_account_balances_task = |blockchain_monitoring_api: M| {
            tokio::task::spawn(async move {
                let accounts: Vec<_> = accounts
                    .into_iter()
                    .flatten()
                    .flat_map(|(network, accounts)| {
                        accounts.into_iter().map(move |account| (network, account))
                    })
                    .collect();

                let balances = accounts.iter().map(|(network, account)| {
                    blockchain_monitoring_api.get_balance(*network, account)
                });
                let balances = join_all(balances).await;

                let balances = accounts.into_iter().zip_eq(balances).collect();

                (blockchain_monitoring_api, balances)
            })
        };

        // TODO: Request balances only when user updates the screen.
        if let Some(balances) = self
            .account_balances_task
            .try_fetch_value_and_rerun(spawn_account_balances_task)
            .await
        {
            self.balances = balances;
        }
    }

    pub async fn deconstruct<L: LedgerApiT, S: StorageApiT>(
        self,
        mut api_registry: ApiRegistry<L, C, M, S>,
    ) -> (StateRegistry, ApiRegistry<L, C, M, S>) {
        api_registry.coin_price_api = Some(self.coin_price_task.abort().await);
        api_registry.blockchain_monitoring_api = Some(self.account_balances_task.abort().await);

        (self.state, api_registry)
    }
}

impl<C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT for Model<C, M> {
    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic().await;

        controller::process_input(event.as_ref()?, self)
    }
}
