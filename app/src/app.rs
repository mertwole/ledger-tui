use std::{io::stdout, marker::PhantomData, time::Duration};

use futures::executor::block_on;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Terminal,
};

use crate::{
    api::{
        blockchain_monitoring::{mock::BlockchainMonitoringApiMock, BlockchainMonitoringApiT},
        cache_utils::ModePlan,
        coin_price::{
            cache::Cache as CoinPriceApiCache, mock::CoinPriceApiMock, CoinPriceApi, CoinPriceApiT,
        },
        common_types::{Account, Network},
        ledger::{
            cache::Cache as LedgerApiCache, mock::LedgerApiMock, Device, DeviceInfo, LedgerApiT,
        },
    },
    screen::{OutgoingMessage, Screen, ScreenName},
};

pub struct App {
    screens: Vec<ScreenName>,
}

// TODO: Add macro to automatically break this registry into sub-registries designated for specific Screens.
pub(crate) struct StateRegistry {
    pub active_device: Option<(Device, DeviceInfo)>,
    pub device_accounts: Option<Vec<(Network, Vec<Account>)>>,
    pub selected_account: Option<(Network, Account)>,
    _phantom: PhantomData<()>,
}

pub(crate) struct ApiRegistry<L, C, M>
where
    L: LedgerApiT,
    C: CoinPriceApiT,
    M: BlockchainMonitoringApiT,
{
    pub ledger_api: L,
    pub coin_price_api: C,
    pub blockchain_monitoring_api: M,
    _phantom: PhantomData<()>,
}

impl StateRegistry {
    fn new() -> StateRegistry {
        StateRegistry {
            active_device: None,
            device_accounts: None,
            selected_account: None,
            _phantom: PhantomData,
        }
    }
}

impl App {
    pub async fn new() -> Self {
        Self {
            screens: vec![ScreenName::Portfolio, ScreenName::DeviceSelection],
        }
    }

    pub async fn run(mut self) {
        stdout().execute(EnterAlternateScreen).unwrap();
        enable_raw_mode().unwrap();
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
        terminal.clear().unwrap();

        self.main_loop(terminal).await;

        stdout().execute(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }

    async fn main_loop<B: Backend>(&mut self, mut terminal: Terminal<B>) {
        let mut state = Some(StateRegistry::new());

        let api_registry = {
            let ledger_api = LedgerApiMock::new(10, 3);
            let mut ledger_api = block_on(LedgerApiCache::new(ledger_api));
            ledger_api.set_all_modes(ModePlan::Transparent);

            let _coin_price_api = CoinPriceApiMock::new();
            let coin_price_api = CoinPriceApi::new("https://data-api.binance.vision");
            let mut coin_price_api = block_on(CoinPriceApiCache::new(coin_price_api));
            coin_price_api.set_all_modes(ModePlan::Slow(Duration::from_secs(1)));

            let mut coin_price_api = block_on(CoinPriceApiCache::new(coin_price_api));
            coin_price_api.set_all_modes(ModePlan::TimedOut(Duration::from_secs(5)));

            let blockchain_monitoring_api = BlockchainMonitoringApiMock::new(4);

            ApiRegistry {
                ledger_api,
                coin_price_api,
                blockchain_monitoring_api,
                _phantom: PhantomData,
            }
        };

        let mut api_registry = Some(api_registry);

        loop {
            let screen = self
                .screens
                .last()
                .expect("At least one screen should be present");

            let screen = Screen::new(*screen, state.take().unwrap(), api_registry.take().unwrap());

            let (new_state, new_api_registry, msg) = Self::screen_loop(screen, &mut terminal);
            state = Some(new_state);
            api_registry = Some(new_api_registry);

            match msg {
                OutgoingMessage::Exit => {
                    return;
                }
                OutgoingMessage::Back => {
                    if self.screens.pop().is_none() {
                        return;
                    }
                }
                OutgoingMessage::SwitchScreen(new_screen) => {
                    self.screens.push(new_screen);
                }
            }
        }
    }

    fn screen_loop<B: Backend, L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
        mut screen: Screen<L, C, M>,
        terminal: &mut Terminal<B>,
    ) -> (StateRegistry, ApiRegistry<L, C, M>, OutgoingMessage) {
        loop {
            terminal.draw(|frame| screen.render(frame)).unwrap();

            let event = event::poll(Duration::ZERO)
                .unwrap()
                .then(|| event::read().unwrap());

            let msg = screen.tick(event);

            if let Some(msg) = msg {
                let (state, api_registry) = screen.deconstruct();
                return (state, api_registry, msg);
            }
        }
    }
}
