use ratatui::{
    crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind},
    layout::Margin,
    style::Color,
    widgets::{Block, BorderType, Borders, List},
    Frame,
};
use std::time::{Duration, Instant};

use crate::api::ledger::{Device, DeviceInfo, LedgerApiT};

const DEVICE_POLL_PERIOD: Duration = Duration::from_secs(2);

pub struct DeviceSelection<L: LedgerApiT> {
    devices: Vec<(Device, DeviceInfo)>,
    previous_poll: Instant,

    ledger_api: L,
}

pub enum OutgoingMessage {
    Quit,
}

impl<L: LedgerApiT> DeviceSelection<L> {
    pub async fn new(ledger_api: L) -> Self {
        Self {
            devices: vec![],
            previous_poll: Instant::now(),
            ledger_api,
        }
    }

    pub async fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.size();

        let block_area = area.inner(Margin::new(8, 8));

        let block = Block::new()
            .border_type(BorderType::Double)
            .borders(Borders::all())
            .border_style(Color::Green);
        let list_area = block.inner(block_area);

        let list = List::new(self.devices.iter().map(|(_, info)| {
            format!(
                "{} MCU v{} SE v{}",
                info.model, info.mcu_version, info.se_version
            )
        }));

        frame.render_widget(block, block_area);
        frame.render_widget(list, list_area);
    }

    pub async fn tick(&mut self) -> Option<OutgoingMessage> {
        if self.previous_poll.elapsed() >= DEVICE_POLL_PERIOD {
            let devices = self.ledger_api.discover_devices().await;
            let mut devices_with_info = Vec::with_capacity(devices.len());

            for device in devices {
                let info = self.ledger_api.get_device_info(&device).await;
                if let Some(info) = info {
                    devices_with_info.push((device, info));
                }
            }

            self.devices = devices_with_info;

            self.previous_poll = Instant::now();
        }

        if !event::poll(Duration::ZERO).unwrap() {
            return None;
        }

        if matches!(
            event::read().unwrap(),
            event::Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code: KeyCode::Char('q'),
                ..
            })
        ) {
            return Some(OutgoingMessage::Quit);
        }

        None
    }
}
