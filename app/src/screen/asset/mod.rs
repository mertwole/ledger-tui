use std::sync::Arc;

use ratatui::{Frame, crossterm::event::Event};
use rust_decimal::Decimal;
use strum::EnumIter;
use tokio::task::JoinHandle;

use super::{OutgoingMessage, ScreenT, resources::Resources};
use crate::{
    api::{
        blockchain_monitoring::{BlockchainMonitoringApiT, TransactionInfo, TransactionUid},
        coin_price::{Coin, CoinPriceApiT, TimePeriod as ApiTimePeriod},
        common_types::Network,
        ledger::LedgerApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

const DEFAULT_SELECTED_TIME_PERIOD: TimePeriod = TimePeriod::Day;

type TransactionList = Vec<(TransactionUid, TransactionInfo)>;
type TaskHandle<R> = Option<JoinHandle<R>>;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    coin_price_history: Option<Vec<PriceHistoryPoint>>,
    transactions: Option<TransactionList>,
    selected_time_period: TimePeriod,
    show_navigation_help: bool,

    state: StateRegistry,
    apis: Arc<ApiRegistry<L, C, M>>,

    price_history_task: TaskHandle<Option<Vec<PriceHistoryPoint>>>,
    transaction_list_task: TaskHandle<TransactionList>,
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

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
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

        let apis = self.apis.clone();
        let spawn_price_history_task = || {
            tokio::task::spawn(async move {
                apis.coin_price_api
                    .get_price_history(coin, Coin::USDT, time_period)
                    .await
            })
        };

        self.price_history_task = Some(match self.price_history_task.take() {
            Some(join_handle) => {
                if join_handle.is_finished() {
                    self.coin_price_history = join_handle.await.unwrap();

                    spawn_price_history_task()
                } else {
                    join_handle
                }
            }
            None => spawn_price_history_task(),
        });

        let apis = self.apis.clone();
        let spawn_transaction_list_task = || {
            tokio::task::spawn(async move {
                let tx_list = apis
                    .blockchain_monitoring_api
                    .get_transactions(selected_network, &selected_account)
                    .await;

                let mut txs = vec![];
                for tx in tx_list {
                    let tx_info = apis
                        .blockchain_monitoring_api
                        .get_transaction_info(selected_network, &tx)
                        .await;

                    txs.push((tx, tx_info));
                }

                txs
            })
        };

        self.transaction_list_task = Some(match self.transaction_list_task.take() {
            Some(join_handle) => {
                if join_handle.is_finished() {
                    self.transactions = Some(join_handle.await.unwrap());

                    spawn_transaction_list_task()
                } else {
                    join_handle
                }
            }
            None => spawn_transaction_list_task(),
        });
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, api_registry: Arc<ApiRegistry<L, C, M>>) -> Self {
        Self {
            coin_price_history: Default::default(),
            transactions: Default::default(),
            selected_time_period: DEFAULT_SELECTED_TIME_PERIOD,
            show_navigation_help: false,

            state,
            apis: api_registry,

            price_history_task: None,
            transaction_list_task: None,
        }
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
