use std::sync::Arc;

use ratatui::{Frame, crossterm::event::Event};
use tokio::task::JoinHandle;

use super::{OutgoingMessage, ScreenT, resources::Resources};
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

type TaskHandle<R> = Option<JoinHandle<R>>;

pub struct Model<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    devices: Vec<(Device, DeviceInfo)>,
    selected_device: Option<usize>,
    show_navigation_help: bool,

    state: StateRegistry,
    apis: Arc<ApiRegistry<L, C, M>>,

    device_list_refresh_task: TaskHandle<Vec<(Device, DeviceInfo)>>,
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Model<L, C, M> {
    async fn tick_logic(&mut self) {
        self.device_list_refresh_task = match self.device_list_refresh_task.take() {
            Some(join_handle) => {
                if join_handle.is_finished() {
                    self.devices = join_handle.await.unwrap();

                    None
                } else {
                    Some(join_handle)
                }
            }
            None => None,
        };

        if self.devices.is_empty() {
            self.selected_device = None;
        }

        if let Some(selected) = self.selected_device.as_mut() {
            if *selected >= self.devices.len() {
                *selected = self.devices.len() - 1;
            }
        }
    }

    fn refresh_device_list(&mut self) {
        let apis = self.apis.clone();

        self.device_list_refresh_task = Some(tokio::task::spawn(async move {
            log::info!("Requesting device list from ledger api");

            let devices = apis.ledger_api.discover_devices().await;
            let mut devices_with_info = Vec::with_capacity(devices.len());

            for device in devices {
                let info = apis.ledger_api.get_device_info(&device).await;
                if let Some(info) = info {
                    devices_with_info.push((device, info));
                }
            }

            log::info!("Discovered {} ledger devices", devices_with_info.len());

            devices_with_info
        }));
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> ScreenT<L, C, M>
    for Model<L, C, M>
{
    fn construct(state: StateRegistry, api_registry: Arc<ApiRegistry<L, C, M>>) -> Self {
        Self {
            devices: vec![],
            selected_device: None,
            show_navigation_help: false,

            state,
            apis: api_registry,

            device_list_refresh_task: None,
        }
    }

    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic().await;

        controller::process_input(event.as_ref()?, self)
    }

    fn deconstruct(self) -> StateRegistry {
        self.state
    }
}
