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

pub struct Model<C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    coin_price_history: Option<Vec<PriceHistoryPoint>>,
    transactions: Option<TransactionList>,
    selected_time_period: TimePeriod,
    show_navigation_help: bool,

    state: StateRegistry,
    apis: ApiSubRegistry<C, M>,

    price_history_task: TaskHandle<Option<Vec<PriceHistoryPoint>>>,
    transaction_list_task: TaskHandle<TransactionList>,
}

struct ApiSubRegistry<C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    coin_price_api: Option<C>,
    blockchain_monitoring_api: Option<M>,
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
    pub fn construct<L: LedgerApiT>(
        state: StateRegistry,
        mut api_registry: ApiRegistry<L, C, M>,
    ) -> (Self, ApiRegistry<L, C, M>) {
        let apis = ApiSubRegistry {
            coin_price_api: api_registry.coin_price_api.take(),
            blockchain_monitoring_api: api_registry.blockchain_monitoring_api.take(),
        };

        (
            Self {
                coin_price_history: Default::default(),
                transactions: Default::default(),
                selected_time_period: DEFAULT_SELECTED_TIME_PERIOD,
                show_navigation_help: false,

                state,
                apis,

                price_history_task: None,
                transaction_list_task: None,
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

        // TODO: Fix.
        let coin_price_api = self.apis.coin_price_api.take().unwrap();
        let spawn_price_history_task = || {
            tokio::task::spawn(async move {
                coin_price_api
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

        // TODO: Fix.
        let blockchain_monitoring_api = self.apis.blockchain_monitoring_api.take().unwrap();
        let spawn_transaction_list_task = || {
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

    pub async fn deconstruct<L: LedgerApiT>(
        mut self,
        mut api_registry: ApiRegistry<L, C, M>,
    ) -> (StateRegistry, ApiRegistry<L, C, M>) {
        api_registry.blockchain_monitoring_api = self.apis.blockchain_monitoring_api.take();
        api_registry.coin_price_api = self.apis.coin_price_api.take();

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
