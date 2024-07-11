use futures::executor::block_on;
use ledger_lib::{
    info::Model, Device as LedgerDevice, Filters, LedgerInfo, LedgerProvider, Transport,
    DEFAULT_TIMEOUT,
};
use std::cell::RefCell;

pub trait LedgerApiT {
    async fn discover_devices(&self) -> Vec<Device>;

    async fn get_device_info(&self, device: &Device) -> Option<DeviceInfo>;
}

#[derive(Clone, Debug)]
pub struct Device {
    info: LedgerInfo,
}

#[derive(Debug)]
pub struct DeviceInfo {
    pub model: String,
    pub se_version: String,
    pub mcu_version: String,
}

pub struct LedgerApi {
    provider: RefCell<LedgerProvider>,
}

impl LedgerApi {
    pub async fn new() -> Self {
        Self {
            provider: RefCell::from(LedgerProvider::init().await),
        }
    }
}

impl LedgerApiT for LedgerApi {
    async fn discover_devices(&self) -> Vec<Device> {
        let devices = block_on(self.provider.borrow_mut().list(Filters::Any)).unwrap();
        devices.into_iter().map(|info| Device { info }).collect()
    }

    async fn get_device_info(&self, device: &Device) -> Option<DeviceInfo> {
        let mut handle = block_on(self.provider.borrow_mut().connect(device.info.clone())).ok()?;

        let info = handle.device_info(DEFAULT_TIMEOUT).await.ok()?;

        let model = match device.info.model {
            Model::NanoS => "Nano S".into(),
            Model::NanoSPlus => "Nano S+".into(),
            Model::NanoX => "Nano X".into(),
            Model::Stax => "Stax".into(),
            Model::Unknown(id) => format!("Unknown {}", id),
        };

        Some(DeviceInfo {
            model,
            se_version: info.se_version,
            mcu_version: info.mcu_version,
        })
    }
}

mod mock {
    use super::*;

    pub struct LedgerApiMock {}

    impl LedgerApiT for LedgerApiMock {
        async fn discover_devices(&self) -> Vec<Device> {
            todo!()
        }

        async fn get_device_info(&self, _device: &Device) -> Option<DeviceInfo> {
            todo!()
        }
    }
}
