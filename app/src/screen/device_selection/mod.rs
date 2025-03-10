use ratatui::{Frame, crossterm::event::Event};

use super::{OutgoingMessage, ScreenT, common::api_task::ApiTask, resources::Resources};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT,
        coin_price::CoinPriceApiT,
        ledger::{Device, DeviceInfo, LedgerApiT},
        storage::StorageApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

mod controller;
mod view;

pub struct Model<L: LedgerApiT> {
    devices: Vec<(Device, DeviceInfo)>,
    selected_device: Option<usize>,
    show_navigation_help: bool,

    state: StateRegistry,

    device_list_refresh_task: ApiTask<L, Vec<(Device, DeviceInfo)>>,
}

impl<L: LedgerApiT> Model<L> {
    pub fn construct<C: CoinPriceApiT, M: BlockchainMonitoringApiT, S: StorageApiT>(
        state: StateRegistry,
        mut api_registry: ApiRegistry<L, C, M, S>,
    ) -> (Self, ApiRegistry<L, C, M, S>) {
        let device_list_refresh_task = ApiTask::new(api_registry.ledger_api.take().unwrap());

        (
            Self {
                devices: vec![],
                selected_device: None,
                show_navigation_help: false,

                state,

                device_list_refresh_task,
            },
            api_registry,
        )
    }

    async fn tick_logic(&mut self) {
        if let Some(devices) = self.device_list_refresh_task.try_fetch_value().await {
            self.devices = devices;
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

    async fn refresh_device_list(&mut self) {
        let spawn_task = |ledger_api: L| {
            tokio::task::spawn(async move {
                log::info!("Requesting device list from ledger api");

                let devices = ledger_api.discover_devices().await;
                let mut devices_with_info = Vec::with_capacity(devices.len());

                for device in devices {
                    let info = ledger_api.get_device_info(&device).await;
                    if let Some(info) = info {
                        devices_with_info.push((device, info));
                    }
                }

                log::info!("Discovered {} ledger devices", devices_with_info.len());

                (ledger_api, devices_with_info)
            })
        };

        self.device_list_refresh_task.run(spawn_task).await;
    }

    pub async fn deconstruct<C: CoinPriceApiT, M: BlockchainMonitoringApiT, S: StorageApiT>(
        self,
        mut api_registry: ApiRegistry<L, C, M, S>,
    ) -> (StateRegistry, ApiRegistry<L, C, M, S>) {
        api_registry.ledger_api = Some(self.device_list_refresh_task.abort().await);

        (self.state, api_registry)
    }
}

impl<L: LedgerApiT> ScreenT for Model<L> {
    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic().await;

        controller::process_input(event.as_ref()?, self).await
    }
}
