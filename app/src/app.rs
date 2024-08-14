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
        common::{Account, Network},
        ledger::{
            cache::Cache as LedgerApiCache, mock::LedgerApiMock, Device, DeviceInfo, LedgerApiT,
        },
    },
    screen::{
        asset::Model as AssetScreen, deposit::Model as DepositScreen,
        device_selection::Model as DeviceSelectionScreen, portfolio::Model as PortfolioScreen,
        OutgoingMessage, Screen, ScreenName,
    },
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
            screens: vec![ScreenName::DeviceSelection],
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

        loop {
            let screen = self
                .screens
                .last()
                .expect("At least one screen should be present");
            let screen = create_screen(*screen);

            let (new_state, msg) = Self::screen_loop(screen, &mut terminal, state.take().unwrap());
            state = Some(new_state);

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

    fn screen_loop<B: Backend>(
        mut screen: Box<dyn Screen>,
        terminal: &mut Terminal<B>,
        state: StateRegistry,
    ) -> (StateRegistry, OutgoingMessage) {
        screen.construct(state);

        loop {
            terminal.draw(|frame| screen.render(frame)).unwrap();

            let event = event::poll(Duration::ZERO)
                .unwrap()
                .then(|| event::read().unwrap());

            let msg = screen.tick(event);

            if let Some(msg) = msg {
                let state = screen.deconstruct();
                return (state, msg);
            }
        }
    }
}

fn create_screen(screen: ScreenName) -> Box<dyn Screen> {
    let ledger_api = LedgerApiMock::new(10, 3);
    let mut ledger_api = block_on(LedgerApiCache::new(ledger_api));
    ledger_api.set_all_modes(ModePlan::Transparent);

    let _coin_price_api = CoinPriceApiMock::new();
    let coin_price_api = CoinPriceApi::new("https://data-api.binance.vision");
    let mut coin_price_api = block_on(CoinPriceApiCache::new(coin_price_api));
    coin_price_api.set_all_modes(ModePlan::TimedOut(Duration::from_secs(5)));

    let blockchain_monitoring_api = BlockchainMonitoringApiMock::new(4);

    let api_registry = ApiRegistry {
        ledger_api,
        coin_price_api,
        blockchain_monitoring_api,
        _phantom: PhantomData,
    };

    match screen {
        ScreenName::Portfolio => Box::from(PortfolioScreen::new(api_registry)),
        ScreenName::DeviceSelection => Box::from(DeviceSelectionScreen::new(api_registry)),
        ScreenName::Asset => Box::from(AssetScreen::new(api_registry)),
        ScreenName::Deposit => Box::from(DepositScreen::new(api_registry)),
    }
}
