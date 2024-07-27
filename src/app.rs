use std::{io::stdout, marker::PhantomData};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Terminal,
};

use crate::{
    api::{
        coin_price::{mock::CoinPriceApiMock, Coin, CoinPriceApiT},
        ledger::{mock::LedgerApiMock, Account, Device, DeviceInfo, Network},
    },
    screen::{
        asset::Model as AssetWindow, device_selection::Model as DeviceSelectionWindow,
        portfolio::Model as PortfolioWindow, OutgoingMessage, Screen, WindowName,
    },
};

pub struct App {}

// TODO: Add macro to automatically break this registry into sub-registries designated for specific windows.
pub(crate) struct StateRegistry {
    pub active_device: Option<(Device, DeviceInfo)>,
    pub device_accounts: Option<Vec<(Network, Vec<Account>)>>,
    pub selected_account: Option<(Network, Account)>,
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
        Self {}
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

        let ledger_api = LedgerApiMock::new(10, 5);
        let coin_price_api = CoinPriceApiMock::new();

        coin_price_api.get_price(Coin::BTC, Coin::USDT).await;

        let mut window: Option<Box<dyn Screen>> =
            Some(Box::from(PortfolioWindow::new(ledger_api, coin_price_api)));

        loop {
            let (new_state, msg) =
                Self::window_loop(window.take().unwrap(), &mut terminal, state.take().unwrap());
            state = Some(new_state);

            match msg {
                OutgoingMessage::Exit => {
                    return;
                }
                OutgoingMessage::SwitchWindow(new_window) => match new_window {
                    WindowName::Portfolio => {
                        let ledger_api = LedgerApiMock::new(10, 5);
                        let coin_price_api = CoinPriceApiMock::new();

                        window = Some(Box::from(PortfolioWindow::new(ledger_api, coin_price_api)));
                    }
                    WindowName::DeviceSelection => {
                        let ledger_api = LedgerApiMock::new(10, 5);
                        window = Some(Box::from(DeviceSelectionWindow::new(ledger_api)));
                    }
                    WindowName::Asset => {
                        let ledger_api = LedgerApiMock::new(10, 5);
                        let coin_price_api = CoinPriceApiMock::new();

                        window = Some(Box::from(AssetWindow::new(ledger_api, coin_price_api)));
                    }
                },
            }
        }
    }

    fn window_loop<B: Backend>(
        mut window: Box<dyn Screen>,
        terminal: &mut Terminal<B>,
        state: StateRegistry,
    ) -> (StateRegistry, OutgoingMessage) {
        window.construct(state);

        loop {
            terminal.draw(|frame| window.render(frame)).unwrap();

            let msg = window.tick();

            if let Some(msg) = msg {
                let state = window.deconstruct();
                return (state, msg);
            }
        }
    }
}
