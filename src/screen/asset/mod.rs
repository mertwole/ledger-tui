use std::time::Instant;

use futures::executor::block_on;
use ratatui::{crossterm::event::Event, Frame};
use rust_decimal::Decimal;

use super::{OutgoingMessage, Screen};
use crate::{
    api::{
        blockchain_monitoring::{BlockchainMonitoringApiT, TransactionInfo, TransactionUid},
        coin_price::{Coin, CoinPriceApiT, TimePeriod as ApiTimePeriod},
        common::Network,
        ledger::LedgerApiT,
    },
    app::StateRegistry,
};

mod controller;
mod view;

const DEFAULT_SELECTED_TIME_PERIOD: TimePeriod = TimePeriod::Day;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    _ledger_api: L, // TODO: Remove it.
    coin_price_api: C,
    blockchain_monitoring_api: M,

    coin_price_history: Option<Vec<PriceHistoryPoint>>,
    transactions: Option<Vec<(TransactionUid, TransactionInfo)>>,
    selected_time_period: TimePeriod,

    state: Option<StateRegistry>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum TimePeriod {
    Day,
    Week,
    Month,
    Year,
    All,
}

struct PriceHistoryPoint {
    timestamp: Instant,
    price: Decimal,
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    pub fn new(ledger_api: L, coin_price_api: C, blockchain_monitoring_api: M) -> Self {
        Self {
            _ledger_api: ledger_api,
            coin_price_api,
            blockchain_monitoring_api,

            coin_price_history: Default::default(),
            transactions: Default::default(),
            selected_time_period: DEFAULT_SELECTED_TIME_PERIOD,

            state: None,
        }
    }

    fn tick_logic(&mut self) {
        let state = self
            .state
            .as_ref()
            .expect("Construct should be called at the start of window lifetime");

        let (selected_network, selected_account) = state
            .selected_account
            .as_ref()
            .expect("Selected account should be present in state"); // TODO: Enforce this rule at `app` level?

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

        let mut history: Vec<_> = block_on(self.coin_price_api.get_price_history(
            coin,
            Coin::USDT,
            time_period,
        ))
        .unwrap()
        .into_iter()
        .map(|(timestamp, price)| PriceHistoryPoint { timestamp, price })
        .collect();
        history.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        self.coin_price_history = Some(history);

        // TODO: Don't make requests to API each tick.
        let tx_list = block_on(
            self.blockchain_monitoring_api
                .get_transactions(*selected_network, selected_account.clone()),
        );
        let txs = tx_list
            .into_iter()
            .map(|tx_uid| {
                (
                    tx_uid.clone(),
                    block_on(
                        self.blockchain_monitoring_api
                            .get_transaction_info(*selected_network, tx_uid),
                    ),
                )
            })
            .collect();

        self.transactions = Some(txs);
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Screen for Model<L, C, M> {
    fn construct(&mut self, state: StateRegistry) {
        self.state = Some(state);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        view::render(self, frame);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic();

        controller::process_input(event.as_ref()?)
    }

    fn deconstruct(self: Box<Self>) -> StateRegistry {
        self.state
            .expect("Construct should be called at the start of window lifetime")
    }
}
