use std::time::{Duration, Instant};

use futures::executor::block_on;
use ratatui::{crossterm::event::Event, Frame};

use super::{OutgoingMessage, Screen};
use crate::{
    api::ledger::{Device, DeviceInfo, LedgerApiT},
    app::StateRegistry,
};

mod controller;
mod view;

const DEVICE_POLL_PERIOD: Duration = Duration::from_secs(2);

pub struct Model<L: LedgerApiT> {
    ledger_api: L,

    devices: Vec<(Device, DeviceInfo)>,
    previous_poll: Instant,
    selected_device: Option<usize>,

    state: Option<StateRegistry>,
}

impl<L: LedgerApiT> Model<L> {
    pub fn new(ledger_api: L) -> Self {
        Self {
            devices: vec![],
            previous_poll: Instant::now() - DEVICE_POLL_PERIOD,
            selected_device: None,
            state: None,
            ledger_api,
        }
    }

    fn tick_logic(&mut self) {
        if self.previous_poll.elapsed() >= DEVICE_POLL_PERIOD {
            let devices = block_on(self.ledger_api.discover_devices());
            let mut devices_with_info = Vec::with_capacity(devices.len());

            for device in devices {
                let info = block_on(self.ledger_api.get_device_info(&device));
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
    }
}

impl<L: LedgerApiT> Screen for Model<L> {
    fn construct(&mut self, state: StateRegistry) {
        self.state = Some(state);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        view::render(self, frame);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic();

        controller::process_input(self, event.as_ref()?)
    }

    fn deconstruct(self: Box<Self>) -> StateRegistry {
        self.state
            .expect("Construct should be called at the start of window lifetime")
    }
}
