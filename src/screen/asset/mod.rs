use std::time::Instant;

use futures::executor::block_on;
use ratatui::{crossterm::event::Event, Frame};
use rust_decimal::Decimal;

use super::{OutgoingMessage, Screen};
use crate::{
    api::{
        blockchain_monitoring::{BlockchainMonitoringApiT, TransactionInfo, TransactionUid},
        coin_price::{Coin, CoinPriceApiT},
        common::Network,
        ledger::LedgerApiT,
    },
    app::StateRegistry,
};

mod controller;
mod view;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    _ledger_api: L, // TODO: Remove it.
    coin_price_api: C,
    blockchain_monitoring_api: M,

    coin_price_history: Option<Vec<(Instant, Decimal)>>,
    transactions: Option<Vec<(TransactionUid, TransactionInfo)>>,

    state: Option<StateRegistry>,
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    pub fn new(ledger_api: L, coin_price_api: C, blockchain_monitoring_api: M) -> Self {
        Self {
            _ledger_api: ledger_api,
            coin_price_api,
            blockchain_monitoring_api,

            coin_price_history: None,
            transactions: Default::default(),

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

        // TODO: Don't make requests to API each tick.
        let mut history =
            block_on(self.coin_price_api.get_price_history(coin, Coin::USDT)).unwrap();
        history.sort_by(|a, b| a.0.cmp(&b.0));

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
