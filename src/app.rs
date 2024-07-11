use std::io::stdout;

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Terminal,
};

use crate::window::{connection_request::ConnectionRequest, ExecutionState, Window};

pub struct App {
    window: Box<dyn Window>,
}

impl App {
    pub async fn new() -> Self {
        let window = Box::from(ConnectionRequest::new());
        Self { window }
    }

    pub async fn run(mut self) {
        stdout().execute(EnterAlternateScreen).unwrap();
        enable_raw_mode().unwrap();
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
        terminal.clear().unwrap();

        loop {
            terminal.draw(|frame| self.window.render(frame)).unwrap();

            match self.window.process_events() {
                ExecutionState::Terminate => break,
                ExecutionState::Continue => {}
                ExecutionState::SwitchWindow(new_window) => {
                    self.window = new_window;
                }
            }
        }

        stdout().execute(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }
}
