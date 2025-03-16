use futures::executor::block_on;
use ratatui::{Frame, crossterm::event::Event};

use super::{OutgoingMessage, ScreenT, common::api_task::ApiTask, resources::Resources};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT,
        coin_price::CoinPriceApiT,
        common_types::{Account, Network},
        ledger::LedgerApiT,
        storage::StorageApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

const ACCOUNT_STORAGE_NAME: &'static str = "accounts";

type AccountList = Vec<(Network, Vec<Account>)>;

pub struct Model<L: LedgerApiT, S: StorageApiT> {
    show_navigation_help: bool,

    state: StateRegistry,

    fetch_accounts_task: ApiTask<L, (Network, Vec<Account>)>,
    store_accounts_task: ApiTask<S, ()>,
}

impl<L: LedgerApiT, S: StorageApiT> Model<L, S> {
    pub fn construct<C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
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
            state.device_accounts = Some(vec![]);
        }

        let fetch_accounts_task = ApiTask::new(api_registry.ledger_api.take().unwrap());
        let store_accounts_task = ApiTask::new(api_registry.storage_api.take().unwrap());

        (
            Self {
                show_navigation_help: false,
                state,
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

    pub async fn deconstruct<C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
        self,
        mut api_registry: ApiRegistry<L, C, M, S>,
    ) -> (StateRegistry, ApiRegistry<L, C, M, S>) {
        api_registry.ledger_api = Some(self.fetch_accounts_task.abort().await);
        api_registry.storage_api = Some(self.store_accounts_task.abort().await);

        (self.state, api_registry)
    }
}

impl<L: LedgerApiT, S: StorageApiT> ScreenT for Model<L, S> {
    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic().await;

        controller::process_input(event.as_ref()?, self).await
    }
}
