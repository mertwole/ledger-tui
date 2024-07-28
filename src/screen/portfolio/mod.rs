use futures::executor::block_on;
use ratatui::Frame;

use super::{OutgoingMessage, Screen};
use crate::{
    api::{
        coin_price::CoinPriceApiT,
        ledger::{LedgerApiT, Network},
    },
    app::StateRegistry,
};

mod controller;
mod view;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT> {
    ledger_api: L,
    coin_price_api: C,

    selected_account: Option<(NetworkIdx, AccountIdx)>,

    state: Option<StateRegistry>,
}

type AccountIdx = usize;
type NetworkIdx = usize;

impl<L: LedgerApiT, C: CoinPriceApiT> Model<L, C> {
    pub fn new(ledger_api: L, coin_price_api: C) -> Self {
        Self {
            ledger_api,
            coin_price_api,
            selected_account: None,
            state: None,
        }
    }

    fn fetch_device_accounts(&mut self) {
        let state = self
            .state
            .as_mut()
            .expect("Construct should be called at the start of window lifetime");

        if state.device_accounts.is_none() {
            if let Some((active_device, _)) = state.active_device.as_ref() {
                // TODO: Load at startup from config and add only on user request.
                // TODO: Filter accounts based on balance.
                state.device_accounts = Some(
                    [Network::Bitcoin, Network::Ethereum]
                        .into_iter()
                        .filter_map(|network| {
                            let accounts: Vec<_> =
                                block_on(self.ledger_api.discover_accounts(active_device, network))
                                    .collect();

                            if accounts.is_empty() {
                                None
                            } else {
                                Some((network, accounts))
                            }
                        })
                        .collect(),
                );
            }
        }
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT> Screen for Model<L, C> {
    fn construct(&mut self, state: StateRegistry) {
        self.state = Some(state);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        view::render(self, frame);
    }

    fn tick(&mut self) -> Option<OutgoingMessage> {
        self.fetch_device_accounts();

        controller::process_input(self)
    }

    fn deconstruct(self: Box<Self>) -> StateRegistry {
        self.state
            .expect("Construct should be called at the start of window lifetime")
    }
}
