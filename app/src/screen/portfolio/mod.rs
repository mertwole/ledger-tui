use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::executor::block_on;
use ratatui::{crossterm::event::Event, Frame};
use rust_decimal::Decimal;

use super::{resources::Resources, OutgoingMessage, ScreenT};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT,
        coin_price::{Coin, CoinPriceApiT},
        common_types::{Account, Network},
        ledger::LedgerApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    selected_account: Option<(NetworkIdx, AccountIdx)>,
    // TODO: Store it in API cache.
    coin_prices: Arc<Mutex<HashMap<Network, Option<Decimal>>>>,
    balances: HashMap<(Network, Account), Decimal>,
    show_navigation_help: bool,

    state: StateRegistry,
    apis: Arc<ApiRegistry<L, C, M>>,
}

type AccountIdx = usize;
type NetworkIdx = usize;

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    fn tick_logic(&mut self) {
        if self.state.device_accounts.is_none() {
            if let Some((active_device, _)) = self.state.active_device.as_ref() {
                // TODO: Load at startup from config and add only on user request.
                // TODO: Filter accounts based on balance.
                self.state.device_accounts = Some(
                    [Network::Bitcoin, Network::Ethereum]
                        .into_iter()
                        .filter_map(|network| {
                            let accounts = block_on(
                                self.apis
                                    .ledger_api
                                    .discover_accounts(active_device, network),
                            );

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

        //let coin_price_api = self.apis.coin_price_api.clone();
        let apis = self.apis.clone();
        let self_coin_prices = self.coin_prices.clone();

        tokio::task::spawn(async move {
            let mut coin_prices = HashMap::new();
            // TODO: Correctly map accounts to coins.
            let networks = [Network::Bitcoin, Network::Ethereum];
            for network in networks {
                let coin = match network {
                    Network::Bitcoin => Coin::BTC,
                    Network::Ethereum => Coin::ETH,
                };

                let price = apis.coin_price_api.get_price(coin, Coin::USDT).await;
                coin_prices.insert(network, price);
            }

            let mut guard = self_coin_prices.lock().expect("Failed to lock mutex");
            *guard = coin_prices;
        });

        // TODO: Don't request balance each tick.
        if let Some(accounts) = self.state.device_accounts.as_ref() {
            for (network, accounts) in accounts {
                for account in accounts {
                    self.balances
                        .entry((*network, account.clone()))
                        .or_insert_with(|| {
                            block_on(
                                self.apis
                                    .blockchain_monitoring_api
                                    .get_balance(*network, account),
                            )
                        });
                }
            }
        }
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, api_registry: ApiRegistry<L, C, M>) -> Self {
        Self {
            selected_account: None,
            coin_prices: Default::default(),
            balances: Default::default(),
            show_navigation_help: false,

            state,
            apis: Arc::from(api_registry),
        }
    }

    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic();

        controller::process_input(event.as_ref()?, self)
    }

    fn deconstruct(self) -> (StateRegistry, ApiRegistry<L, C, M>) {
        (self.state, todo!())
    }
}
