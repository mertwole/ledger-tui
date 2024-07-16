use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Alignment, Margin},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, List, Padding},
    Frame,
};
use std::time::{Duration, Instant};

use crate::api::ledger::{Device, DeviceInfo, LedgerApiT};

use super::EventExt;

const DEVICE_POLL_PERIOD: Duration = Duration::from_secs(2);

pub struct DeviceSelection<L: LedgerApiT> {
    devices: Vec<(Device, DeviceInfo)>,
    previous_poll: Instant,

    selected_device: Option<usize>,

    ledger_api: L,
}

pub enum OutgoingMessage {
    Quit,
}

impl<L: LedgerApiT> DeviceSelection<L> {
    pub async fn new(ledger_api: L) -> Self {
        Self {
            devices: vec![],
            previous_poll: Instant::now() - DEVICE_POLL_PERIOD,
            selected_device: None,
            ledger_api,
        }
    }

    pub async fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.size();

        let list_block = Block::new()
            .border_type(BorderType::Double)
            .borders(Borders::all())
            .border_style(Color::Green)
            .padding(Padding::uniform(1))
            .title("Select a device")
            .title_alignment(Alignment::Center);

        let mut list_height = 0;
        let list = List::new(self.devices.iter().enumerate().map(|(idx, (_, info))| {
            let label = format!(
                "{} MCU v{} SE v{}",
                info.model, info.mcu_version, info.se_version
            );

            let mut item = Text::centered(label.into());

            if Some(idx) == self.selected_device {
                item = item.bold().bg(Color::DarkGray);
            }

            list_height += item.height();

            item
        }));

        let list_area = list_block.inner(area);
        let margin = list_area.height.saturating_sub(list_height as u16) / 2;
        let list_area = list_area.inner(Margin::new(0, margin));

        frame.render_widget(list_block, area);
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

        if self.devices.is_empty() {
            self.selected_device = None;
        }

        if let Some(selected) = self.selected_device.as_mut() {
            if *selected >= self.devices.len() {
                *selected = self.devices.len() - 1;
            }
        }

        self.process_input().await
    }

    async fn process_input(&mut self) -> Option<OutgoingMessage> {
        if !event::poll(Duration::ZERO).unwrap() {
            return None;
        }

        let event = event::read().unwrap();

        if event.is_key_pressed(KeyCode::Down) && !self.devices.is_empty() {
            if let Some(selected) = self.selected_device.as_mut() {
                *selected = (self.devices.len() - 1).min(*selected + 1);
            } else {
                self.selected_device = Some(0);
            }
        }

        if event.is_key_pressed(KeyCode::Up) && !self.devices.is_empty() {
            if let Some(selected) = self.selected_device.as_mut() {
                *selected = if *selected == 0 { 0 } else { *selected - 1 };
            } else {
                self.selected_device = Some(self.devices.len() - 1);
            }
        }

        if event.is_key_pressed(KeyCode::Char('q')) {
            return Some(OutgoingMessage::Quit);
        }

        None
    }
}
