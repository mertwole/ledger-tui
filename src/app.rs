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
        coin_price::mock::CoinPriceApiMock,
        ledger::{mock::LedgerApiMock, Account, Device, DeviceInfo, Network},
    },
    screen::{
        asset::Model as AssetWindow, device_selection::Model as DeviceSelectionWindow,
        portfolio::Model as PortfolioWindow, OutgoingMessage, Screen, ScreenName,
    },
};

pub struct App {
    screens: Vec<ScreenName>,
}

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
        Self {
            screens: vec![ScreenName::Portfolio],
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

            let msg = screen.tick();

            if let Some(msg) = msg {
                let state = screen.deconstruct();
                return (state, msg);
            }
        }
    }
}

fn create_screen(screen: ScreenName) -> Box<dyn Screen> {
    let ledger_api = LedgerApiMock::new(10, 5);
    let coin_price_api = CoinPriceApiMock::new();

    match screen {
        ScreenName::Portfolio => Box::from(PortfolioWindow::new(ledger_api, coin_price_api)),
        ScreenName::DeviceSelection => Box::from(DeviceSelectionWindow::new(ledger_api)),
        ScreenName::Asset => Box::from(AssetWindow::new(ledger_api, coin_price_api)),
    }
}
