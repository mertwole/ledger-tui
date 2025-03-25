use std::collections::HashMap;

use bigdecimal::BigDecimal;
use futures::{executor::block_on, future::join_all};
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

const ACCOUNT_STORAGE_NAME: &str = "accounts.json";

type AccountList = Vec<(Network, Vec<Account>)>;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT, S: StorageApiT> {
    selected_network: Option<NetworkIdx>,
    selected_account: Option<AccountIdx>,
    coin_prices: HashMap<Network, Option<Decimal>>,
    balances: HashMap<(Network, Account), BigDecimal>,
    show_navigation_help: bool,

    state: StateRegistry,

    coin_price_task: ApiTask<C, HashMap<Network, Option<Decimal>>>,
    account_balances_task: ApiTask<M, HashMap<(Network, Account), BigDecimal>>,
    fetch_accounts_task: ApiTask<L, (Network, Vec<Account>)>,
    store_accounts_task: ApiTask<S, ()>,
}

type AccountIdx = usize;
type NetworkIdx = usize;

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT, S: StorageApiT>
    Model<L, C, M, S>
{
    pub fn construct(
        mut state: StateRegistry,
        mut api_registry: ApiRegistry<L, C, M, S>,
    ) -> (Self, ApiRegistry<L, C, M, S>) {
        // TODO: Store unique device id alongside with accounts to have ability
        // to use more than one device with an app.
        let accounts = block_on(
            api_registry
                .storage_api
                .as_mut()
                .unwrap()
                .load(ACCOUNT_STORAGE_NAME),
        );

        if let Some(accounts) = accounts {
            let accounts: AccountList = serde_json::from_str(&accounts).unwrap();
            state.device_accounts = Some(accounts);
        } else {
            // TODO: Make it empty and add networks only on user request.
            state.device_accounts = Some(vec![
                (Network::Bitcoin, vec![]),
                (Network::Ethereum, vec![]),
            ]);
        }

        let coin_price_task = ApiTask::new(api_registry.coin_price_api.take().unwrap());
        let account_balances_task =
            ApiTask::new(api_registry.blockchain_monitoring_api.take().unwrap());
        let fetch_accounts_task = ApiTask::new(api_registry.ledger_api.take().unwrap());
        let store_accounts_task = ApiTask::new(api_registry.storage_api.take().unwrap());

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
                fetch_accounts_task,
                store_accounts_task,
            },
            api_registry,
        )
    }

    async fn tick_logic(&mut self) {
        if let Some((network, accounts)) = self.fetch_accounts_task.try_fetch_value().await {
            let device_accounts = self.state.device_accounts.as_mut().unwrap();
            let idx = device_accounts.iter().position(|(nw, _)| *nw == network);
            if let Some(idx) = idx {
                device_accounts[idx] = (network, accounts);
            } else {
                device_accounts.push((network, accounts));
            }

            let device_accounts = device_accounts.clone();
            let spawn_store_task = |mut storage_api: S| {
                tokio::task::spawn(async move {
                    let data = serde_json::to_string(&device_accounts).unwrap();
                    storage_api.save(ACCOUNT_STORAGE_NAME, data).await;

                    (storage_api, ())
                })
            };

            self.store_accounts_task.run(spawn_store_task).await;
        }

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

    async fn fetch_accounts(&mut self, network: Network) {
        let active_device = self
            .state
            .active_device
            .clone()
            .expect("TODO: Enforce this rule at app level?")
            .0;

        let spawn_task = |ledger_api: L| {
            tokio::task::spawn(async move {
                ledger_api.open_app(&active_device, network).await;

                let accounts = ledger_api.discover_accounts(&active_device, network).await;

                (ledger_api, (network, accounts))
            })
        };

        self.fetch_accounts_task.run(spawn_task).await;
    }

    pub async fn deconstruct(
        self,
        mut api_registry: ApiRegistry<L, C, M, S>,
    ) -> (StateRegistry, ApiRegistry<L, C, M, S>) {
        api_registry.coin_price_api = Some(self.coin_price_task.abort().await);
        api_registry.blockchain_monitoring_api = Some(self.account_balances_task.abort().await);
        api_registry.ledger_api = Some(self.fetch_accounts_task.abort().await);
        api_registry.storage_api = Some(self.store_accounts_task.abort().await);

        (self.state, api_registry)
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT, S: StorageApiT> ScreenT
    for Model<L, C, M, S>
{
    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic().await;

        controller::process_input(event.as_ref()?, self).await
    }
}
