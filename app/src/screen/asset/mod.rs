use ratatui::{Frame, crossterm::event::Event};
use rust_decimal::Decimal;
use strum::EnumIter;

use super::{OutgoingMessage, ScreenT, common::api_task::ApiTask, resources::Resources};
use crate::{
    api::{
        blockchain_monitoring::{BlockchainMonitoringApiT, TransactionInfo, TransactionUid},
        coin_price::{Coin, CoinPriceApiT, TimePeriod as ApiTimePeriod},
        common_types::Network,
        ledger::LedgerApiT,
        storage::StorageApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

const DEFAULT_SELECTED_TIME_PERIOD: TimePeriod = TimePeriod::Day;

type TransactionList = Vec<(TransactionUid, TransactionInfo)>;

pub struct Model<C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    coin_price_history: Option<Vec<PriceHistoryPoint>>,
    transactions: Option<TransactionList>,
    selected_time_period: TimePeriod,
    show_navigation_help: bool,

    state: StateRegistry,

    price_history_task: ApiTask<C, Option<Vec<PriceHistoryPoint>>>,
    transaction_list_task: ApiTask<M, TransactionList>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
enum TimePeriod {
    Day,
    Week,
    Month,
    Year,
    All,
}

type PriceHistoryPoint = Decimal;

impl<C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<C, M> {
    pub fn construct<L: LedgerApiT, S: StorageApiT>(
        state: StateRegistry,
        mut api_registry: ApiRegistry<L, C, M, S>,
    ) -> (Self, ApiRegistry<L, C, M, S>) {
        let price_history_task = ApiTask::new(api_registry.coin_price_api.take().unwrap());
        let transaction_list_task =
            ApiTask::new(api_registry.blockchain_monitoring_api.take().unwrap());

        (
            Self {
                coin_price_history: Default::default(),
                transactions: Default::default(),
                selected_time_period: DEFAULT_SELECTED_TIME_PERIOD,
                show_navigation_help: false,

                state,

                price_history_task,
                transaction_list_task,
            },
            api_registry,
        )
    }

    async fn tick_logic(&mut self) {
        let (selected_network, selected_account) = self
            .state
            .selected_account
            .as_ref()
            .expect("Selected account should be present in state")
            .clone(); // TODO: Enforce this rule at `app` level?

        let coin = match selected_network {
            Network::Bitcoin => Coin::BTC,
            Network::Ethereum => Coin::ETH,
        };

        let time_period = match self.selected_time_period {
            TimePeriod::Day => ApiTimePeriod::Day,
            TimePeriod::Week => ApiTimePeriod::Week,
            TimePeriod::Month => ApiTimePeriod::Month,
            TimePeriod::Year => ApiTimePeriod::Year,
            TimePeriod::All => ApiTimePeriod::All,
        };

        let spawn_price_history_task = |coin_price_api: C| {
            tokio::task::spawn(async move {
                let result = coin_price_api
                    .get_price_history(coin, Coin::USDT, time_period)
                    .await;

                (coin_price_api, result)
            })
        };

        if let Some(price_history) = self
            .price_history_task
            .try_fetch_value_and_rerun(spawn_price_history_task)
            .await
        {
            self.coin_price_history = price_history;
        }

        let spawn_transaction_list_task = |blockchain_monitoring_api: M| {
            tokio::task::spawn(async move {
                let tx_list = blockchain_monitoring_api
                    .get_transactions(selected_network, &selected_account)
                    .await;

                let mut txs = vec![];
                for tx in tx_list {
                    let tx_info = blockchain_monitoring_api
                        .get_transaction_info(selected_network, &tx)
                        .await;

                    txs.push((tx, tx_info));
                }

                (blockchain_monitoring_api, txs)
            })
        };

        if let Some(transactions) = self
            .transaction_list_task
            .try_fetch_value_and_rerun(spawn_transaction_list_task)
            .await
        {
            self.transactions = Some(transactions);
        }
    }

    pub async fn deconstruct<L: LedgerApiT, S: StorageApiT>(
        self,
        mut api_registry: ApiRegistry<L, C, M, S>,
    ) -> (StateRegistry, ApiRegistry<L, C, M, S>) {
        api_registry.coin_price_api = Some(self.price_history_task.abort().await);
        api_registry.blockchain_monitoring_api = Some(self.transaction_list_task.abort().await);

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
