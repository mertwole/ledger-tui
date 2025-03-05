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

pub struct Model<L: LedgerApiT> {
    devices: Vec<(Device, DeviceInfo)>,
    selected_device: Option<usize>,
    show_navigation_help: bool,

    state: StateRegistry,
    apis: ApiSubRegistry<L>,

    device_list_refresh_task: TaskHandle<Vec<(Device, DeviceInfo)>>,
}

struct ApiSubRegistry<L: LedgerApiT> {
    ledger_api: Option<L>,
}

impl<L: LedgerApiT> Model<L> {
    pub fn construct<C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
        state: StateRegistry,
        mut api_registry: ApiRegistry<L, C, M>,
    ) -> (Self, ApiRegistry<L, C, M>) {
        let apis = ApiSubRegistry {
            ledger_api: api_registry.ledger_api.take(),
        };

        (
            Self {
                devices: vec![],
                selected_device: None,
                show_navigation_help: false,

                state,
                apis,

                device_list_refresh_task: None,
            },
            api_registry,
        )
    }

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
        // TODO: Fix.
        let ledger_api = self.apis.ledger_api.take().unwrap();
        self.device_list_refresh_task = Some(tokio::task::spawn(async move {
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

            devices_with_info
        }));
    }

    pub async fn deconstruct<C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
        mut self,
        mut api_registry: ApiRegistry<L, C, M>,
    ) -> (StateRegistry, ApiRegistry<L, C, M>) {
        api_registry.ledger_api = self.apis.ledger_api.take();

        (self.state, api_registry)
    }
}

impl<L: LedgerApiT> ScreenT for Model<L> {
    fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        view::render(self, frame, resources);
    }

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        self.tick_logic().await;

        controller::process_input(event.as_ref()?, self)
    }
}
