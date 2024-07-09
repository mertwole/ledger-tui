use futures::executor::block_on;
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Alignment, Margin},
    style::Color,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::device::Device;

use super::{portfolio::Protfolio, ExecutionState, Window};

pub struct ConnectionRequest {}

impl ConnectionRequest {
    pub fn new() -> Self {
        Self {}
    }
}

impl Window for ConnectionRequest {
    fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.size();

        let block_area = area.inner(Margin::new(8, 8));

        let block = Block::new()
            .border_type(BorderType::Double)
            .borders(Borders::all())
            .border_style(Color::Green);
        let label_area = block.inner(block_area);

        let label =
            Paragraph::new("Ledger is not discovered. Connect your device and try again(c)")
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

        frame.render_widget(block, block_area);
        frame.render_widget(label, label_area);
    }

    fn process_events(&mut self) -> ExecutionState {
        if event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    return ExecutionState::Terminate;
                }

                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('c') {
                    let device = block_on(Device::discover());
                    if let Some(device) = device {
                        let portfolio = Protfolio::new(device);
                        return ExecutionState::SwitchWindow(Box::from(portfolio));
                    } else {
                        return ExecutionState::Continue;
                    }
                }
            }
        }

        ExecutionState::Continue
    }
}
