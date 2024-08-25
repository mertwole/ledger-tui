use std::time::{Duration, Instant};

use futures::executor::block_on;
use ratatui::{crossterm::event::Event, Frame};

use super::{resources::Resources, OutgoingMessage, ScreenT};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT,
        coin_price::CoinPriceApiT,
        ledger::{Device, DeviceInfo, LedgerApiT},
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

const DEVICE_POLL_PERIOD: Duration = Duration::from_secs(2);

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    devices: Vec<(Device, DeviceInfo)>,
    previous_poll: Instant,
    selected_device: Option<usize>,

    state: StateRegistry,
    apis: ApiRegistry<L, C, M>,
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    fn tick_logic(&mut self) {
        if self.previous_poll.elapsed() >= DEVICE_POLL_PERIOD {
            let devices = block_on(self.apis.ledger_api.discover_devices());
            let mut devices_with_info = Vec::with_capacity(devices.len());

            for device in devices {
                let info = block_on(self.apis.ledger_api.get_device_info(&device));
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

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, api_registry: ApiRegistry<L, C, M>) -> Self {
        Self {
            devices: vec![],
            previous_poll: Instant::now() - DEVICE_POLL_PERIOD,
            selected_device: None,

            state,
            apis: api_registry,
        }
    }

    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic();

        controller::process_input(self, event.as_ref()?)
    }

    fn deconstruct(self) -> (StateRegistry, ApiRegistry<L, C, M>) {
        (self.state, self.apis)
    }
}
