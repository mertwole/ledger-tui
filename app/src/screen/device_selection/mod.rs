use std::sync::{Arc, Mutex};

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

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    devices: Arc<Mutex<Vec<(Device, DeviceInfo)>>>,
    selected_device: Option<usize>,
    show_navigation_help: bool,

    state: StateRegistry,
    apis: Arc<ApiRegistry<L, C, M>>,
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    fn tick_logic(&mut self) {
        let devices = self
            .devices
            .lock()
            .expect("Failed to acquire lock on mutex");

        if devices.is_empty() {
            self.selected_device = None;
        }

        if let Some(selected) = self.selected_device.as_mut() {
            if *selected >= devices.len() {
                *selected = devices.len() - 1;
            }
        }
    }

    fn refresh_device_list(&self) {
        let state_devices = self.devices.clone();
        let apis = self.apis.clone();

        tokio::task::spawn(async move {
            let devices = apis.ledger_api.discover_devices().await;
            let mut devices_with_info = Vec::with_capacity(devices.len());

            for device in devices {
                let info = apis.ledger_api.get_device_info(&device).await;
                if let Some(info) = info {
                    devices_with_info.push((device, info));
                }
            }

            *state_devices
                .lock()
                .expect("Failed to acquire lock on mutex") = devices_with_info;
        });
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, api_registry: Arc<ApiRegistry<L, C, M>>) -> Self {
        Self {
            devices: Arc::new(Mutex::new(vec![])),
            selected_device: None,
            show_navigation_help: false,

            state,
            apis: api_registry,
        }
    }

    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic();

        controller::process_input(event.as_ref()?, self)
    }

    fn deconstruct(self) -> StateRegistry {
        self.state
    }
}
