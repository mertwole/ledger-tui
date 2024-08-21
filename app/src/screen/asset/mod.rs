use futures::executor::block_on;
use ratatui::{crossterm::event::Event, Frame};
use rust_decimal::Decimal;
use strum::EnumIter;

use super::{OutgoingMessage, ScreenT};
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

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    coin_price_history: Option<Vec<PriceHistoryPoint>>,
    transactions: Option<Vec<(TransactionUid, TransactionInfo)>>,
    selected_time_period: TimePeriod,

    state: StateRegistry,
    apis: ApiRegistry<L, C, M>,
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

        self.coin_price_history = block_on(self.apis.coin_price_api.get_price_history(
            coin,
            Coin::USDT,
            time_period,
        ));

        // TODO: Don't make requests to API each tick.
        let tx_list = block_on(
            self.apis
                .blockchain_monitoring_api
                .get_transactions(*selected_network, selected_account),
        );
        let txs = tx_list
            .into_iter()
            .map(|tx_uid| {
                (
                    tx_uid.clone(),
                    block_on(
                        self.apis
                            .blockchain_monitoring_api
                            .get_transaction_info(*selected_network, &tx_uid),
                    ),
                )
            })
            .collect();

        self.transactions = Some(txs);
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, api_registry: ApiRegistry<L, C, M>) -> Self {
        Self {
            coin_price_history: Default::default(),
            transactions: Default::default(),
            selected_time_period: DEFAULT_SELECTED_TIME_PERIOD,

            state,
            apis: api_registry,
        }
    }

    fn render(&self, frame: &mut Frame<'_>) {
        view::render(self, frame);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic();

        controller::process_input(event.as_ref()?, self)
    }

    fn deconstruct(self) -> (StateRegistry, ApiRegistry<L, C, M>) {
        (self.state, self.apis)
    }
}
