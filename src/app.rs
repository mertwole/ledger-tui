use std::{collections::HashMap, io::stdout, marker::PhantomData};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Terminal,
};

use crate::{
    api::ledger::{mock::LedgerApiMock, Account, Device, DeviceInfo, Network},
    window::{
        device_selection::DeviceSelection, portfolio::Portfolio, OutgoingMessage, Window,
        WindowName,
    },
};

pub struct App {}

// TODO: Add macro to automatically break this registry into sub-registries designated for specific windows.
pub(crate) struct StateRegistry {
    pub active_device: Option<(Device, DeviceInfo)>,
    pub device_accounts: Option<HashMap<Network, Vec<Account>>>,
    _phantom: PhantomData<()>,
}

impl StateRegistry {
    fn new() -> StateRegistry {
        StateRegistry {
            active_device: None,
            device_accounts: None,
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
        let mut window: Option<Box<dyn Window>> = Some(Box::from(Portfolio::new(ledger_api)));

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
                        window = Some(Box::from(Portfolio::new(ledger_api)));
                    }
                    WindowName::DeviceSelection => {
                        let ledger_api = LedgerApiMock::new(10, 5);
                        window = Some(Box::from(DeviceSelection::new(ledger_api)));
                    }
                },
            }
        }
    }

    fn window_loop<B: Backend>(
        mut window: Box<dyn Window>,
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
