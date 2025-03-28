use std::time::Instant;

use bigdecimal::{BigDecimal, FromPrimitive};
use ratatui::{Frame, crossterm::event::Event};

use super::{OutgoingMessage, ScreenT, common::api_task::ApiTask, resources::Resources};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        ledger::LedgerApiT, storage::StorageApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

type SignedTx = Vec<u8>;

pub struct Model<L: LedgerApiT> {
    show_navigation_help: bool,
    receiver_address: Option<String>,
    send_amount: Option<BigDecimal>,

    state: StateRegistry,

    sign_tx_task: ApiTask<L, SignedTx>,
}

impl<L: LedgerApiT> Model<L> {
    pub fn construct<C: CoinPriceApiT, M: BlockchainMonitoringApiT, S: StorageApiT>(
        state: StateRegistry,
        mut api_registry: ApiRegistry<L, C, M, S>,
    ) -> (Self, ApiRegistry<L, C, M, S>) {
        let sign_tx_task = ApiTask::new(api_registry.ledger_api.take().unwrap());

        (
            Self {
                show_navigation_help: false,
                receiver_address: None,
                send_amount: None,

                state,

                sign_tx_task,
            },
            api_registry,
        )
    }

    pub async fn deconstruct<C: CoinPriceApiT, M: BlockchainMonitoringApiT, S: StorageApiT>(
        self,
        mut api_registry: ApiRegistry<L, C, M, S>,
    ) -> (StateRegistry, ApiRegistry<L, C, M, S>) {
        api_registry.ledger_api = Some(self.sign_tx_task.abort().await);

        (self.state, api_registry)
    }
}

impl<L: LedgerApiT> ScreenT for Model<L> {
    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        controller::process_input(event.as_ref()?, self)
    }
}
