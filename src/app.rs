use futures::executor::block_on;
use std::io::stdout;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Terminal,
};

use crate::{
    api::ledger::mock::LedgerApiMock,
    window::device_selection::{DeviceSelection, OutgoingMessage},
};

pub struct App {}

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
        let ledger_api = LedgerApiMock::new(10, 3);

        let mut window = DeviceSelection::new(ledger_api).await;

        loop {
            terminal
                .draw(|frame| block_on(window.render(frame)))
                .unwrap();

            if let Some(OutgoingMessage::Quit) = window.tick().await {
                break;
            }
        }
    }
}
