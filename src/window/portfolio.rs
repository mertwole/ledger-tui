use std::sync::mpsc::{channel, Receiver};

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Alignment, Margin},
    style::Color,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::device::Device;

use super::{ExecutionState, Window};

pub struct Protfolio {
    device_info: Receiver<String>,
    display_info: String,
}

impl Protfolio {
    pub fn new(device: Device) -> Self {
        let device_info = start_poll_device_info(device);

        Self {
            device_info,
            display_info: String::new(),
        }
    }
}

impl Window for Protfolio {
    fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.size();

        let block_area = area.inner(Margin::new(8, 8));

        let block = Block::new()
            .border_type(BorderType::Double)
            .borders(Borders::all())
            .border_style(Color::Yellow);
        let label_area = block.inner(block_area);

        let label = Paragraph::new(format!("Connected to ledger:\n{}", self.display_info))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(block, block_area);
        frame.render_widget(label, label_area);
    }

    fn process_events(&mut self) -> ExecutionState {
        for info in self.device_info.try_iter() {
            self.display_info = info;
        }

        if event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    return ExecutionState::Terminate;
                }
            }
        }

        ExecutionState::Continue
    }
}

fn start_poll_device_info(mut device: Device) -> Receiver<String> {
    let (sender, receiver) = channel();

    tokio::spawn(async move {
        loop {
            let info = device.get_info().await;
            sender.send(info).unwrap();
        }
    });

    receiver
}
