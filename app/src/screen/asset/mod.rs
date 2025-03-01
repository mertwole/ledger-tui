use std::sync::{Arc, Mutex};

use ratatui::{Frame, crossterm::event::Event};
use rust_decimal::Decimal;
use strum::EnumIter;

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

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    coin_price_history: Arc<Mutex<Option<Vec<PriceHistoryPoint>>>>,
    transactions: Arc<Mutex<Option<TransactionList>>>,
    selected_time_period: TimePeriod,
    show_navigation_help: bool,

    state: StateRegistry,
    apis: Arc<ApiRegistry<L, C, M>>,
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
    fn tick_logic(&mut self) {
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
        let coin_price_history = self.coin_price_history.clone();

        tokio::task::spawn(async move {
            let price_history = apis
                .coin_price_api
                .get_price_history(coin, Coin::USDT, time_period)
                .await;

            *coin_price_history
                .lock()
                .expect("Failed to acquire lock on mutex") = price_history;
        });

        let apis = self.apis.clone();
        let transactions = self.transactions.clone();

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

            *transactions
                .lock()
                .expect("Failed to acquire lock on mutex") = Some(txs);
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
        }
    }

    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic();

        controller::process_input(event.as_ref()?, self)
    }

    fn deconstruct(self) -> StateRegistry {
        self.state
    }
}
