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

        let model = model_to_string(&device.info.model);

        Some(DeviceInfo {
            model,
            se_version: info.se_version,
            mcu_version: info.mcu_version,
        })
    }
}

fn model_to_string(model: &Model) -> String {
    match model {
        Model::NanoS => "Nano S".into(),
        Model::NanoSPlus => "Nano S+".into(),
        Model::NanoX => "Nano X".into(),
        Model::Stax => "Stax".into(),
        Model::Unknown(id) => format!("Unknown {}", id),
    }
}

pub mod mock {
    use ledger_lib::{info::ConnInfo, transport::UsbInfo};
    use std::iter;

    use super::*;

    pub struct LedgerApiMock {
        devices: Vec<Device>,
    }

    impl LedgerApiMock {
        pub fn new(device_count: usize) -> Self {
            let conn = ConnInfo::Usb(UsbInfo {
                vid: 0,
                pid: 0,
                path: None,
            });

            let devices = vec![
                Device {
                    info: LedgerInfo {
                        model: Model::NanoX,
                        conn: conn.clone(),
                    },
                },
                Device {
                    info: LedgerInfo {
                        model: Model::NanoSPlus,
                        conn: conn.clone(),
                    },
                },
                Device {
                    info: LedgerInfo {
                        model: Model::Stax,
                        conn: conn.clone(),
                    },
                },
                Device {
                    info: LedgerInfo {
                        model: Model::NanoS,
                        conn: conn.clone(),
                    },
                },
            ];

            let devices = iter::repeat(devices).flatten().take(device_count).collect();

            Self { devices }
        }
    }

    impl LedgerApiT for LedgerApiMock {
        async fn discover_devices(&self) -> Vec<Device> {
            self.devices.clone()
        }

        async fn get_device_info(&self, device: &Device) -> Option<DeviceInfo> {
            Some(DeviceInfo {
                model: model_to_string(&device.info.model),
                se_version: "0.0.0".into(),
                mcu_version: "0.0".into(),
            })
        }
    }
}
