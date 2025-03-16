use std::marker::PhantomData;

use futures::executor::block_on;
use ratatui::{Frame, crossterm::event::Event};

use super::{OutgoingMessage, ScreenT, resources::Resources};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        common_types::Network, ledger::LedgerApiT, storage::StorageApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

pub struct Model<L: LedgerApiT, S: StorageApiT> {
    show_navigation_help: bool,

    state: StateRegistry,

    _phantom: PhantomData<(L, S)>,
}

impl<L: LedgerApiT, S: StorageApiT> Model<L, S> {
    pub fn construct<C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
        mut state: StateRegistry,
        api_registry: ApiRegistry<L, C, M, S>,
    ) -> (Self, ApiRegistry<L, C, M, S>) {
        let active_device = state
            .active_device
            .clone()
            .expect("TODO: Enforce this rule at app level?")
            .0;

        block_on(
            api_registry
                .ledger_api
                .as_ref()
                .unwrap()
                .open_app(&active_device, Network::Bitcoin),
        );

        let mut device_accounts = vec![];
        for network in [Network::Bitcoin, Network::Ethereum] {
            let accounts = block_on(
                api_registry
                    .ledger_api
                    .as_ref()
                    .unwrap()
                    .discover_accounts(&active_device, network),
            );

            if !accounts.is_empty() {
                device_accounts.push((network, accounts));
            }
        }

        state.device_accounts = Some(device_accounts);

        (
            Self {
                show_navigation_help: false,
                state,
                _phantom: PhantomData,
            },
            api_registry,
        )
    }

    async fn tick_logic(&mut self) {}

    pub async fn deconstruct<C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
        self,
        api_registry: ApiRegistry<L, C, M, S>,
    ) -> (StateRegistry, ApiRegistry<L, C, M, S>) {
        (self.state, api_registry)
    }
}

impl<L: LedgerApiT, S: StorageApiT> ScreenT for Model<L, S> {
    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic().await;

        controller::process_input(event.as_ref()?, self)
    }
}
