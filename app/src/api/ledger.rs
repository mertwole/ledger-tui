use std::{cell::RefCell, hash::Hash, sync::Mutex};

use api_proc_macro::implement_cache;
use async_trait::async_trait;
use futures::executor::block_on;
use ledger_lib::{
    info::Model, Device as LedgerDevice, Filters, LedgerInfo, LedgerProvider, Transport,
    DEFAULT_TIMEOUT,
};

use super::common_types::{Account, Network};

implement_cache!(
    #[async_trait]
    pub trait LedgerApiT: Send + Sync {
        async fn discover_devices(&self) -> Vec<Device>;

        async fn get_device_info(&self, device: &Device) -> Option<DeviceInfo>;

        // TODO: Return stream of accounts?
        async fn discover_accounts(&self, device: &Device, network: Network) -> Vec<Account>;
    }
);

#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    info: LedgerInfo,
}

impl Eq for Device {}

impl Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.info.model.to_string().hash(state);
        format!("{}", self.info.conn).hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub model: String,
    pub se_version: String,
    pub mcu_version: String,
}

pub struct LedgerApi {
    provider: Mutex<LedgerProvider>,
}

impl LedgerApi {
    pub async fn new() -> Self {
        Self {
            provider: Mutex::from(LedgerProvider::init().await),
        }
    }
}

#[async_trait]
impl LedgerApiT for LedgerApi {
    async fn discover_devices(&self) -> Vec<Device> {
        let devices = block_on(
            self.provider
                .lock()
                .expect("Failed to acquire lock on mutex")
                .list(Filters::Any),
        )
        .unwrap();
        devices.into_iter().map(|info| Device { info }).collect()
    }

    async fn get_device_info(&self, device: &Device) -> Option<DeviceInfo> {
        let mut handle = block_on(
            self.provider
                .lock()
                .expect("Failed to acquire lock on mutex")
                .connect(device.info.clone()),
        )
        .ok()?;

        let info = handle.device_info(DEFAULT_TIMEOUT).await.ok()?;

        let model = model_to_string(&device.info.model);

        Some(DeviceInfo {
            model,
            se_version: info.se_version,
            mcu_version: info.mcu_version,
        })
    }

    async fn discover_accounts(&self, _device: &Device, _network: Network) -> Vec<Account> {
        todo!()
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
    use std::{collections::HashMap, iter};

    use super::*;

    pub struct LedgerApiMock {
        devices: Vec<Device>,
        accounts: HashMap<Network, Vec<Account>>,
    }

    impl LedgerApiMock {
        pub fn new(device_count: usize, account_count: usize) -> Self {
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

            let btc_accounts = vec![
                Account {
                    pk: "0x0123456789012345678901234567890101234567890123456789012345678901".into(),
                },
                Account {
                    pk: "0x9876543210987654321098765432109876543210987654321098765432109876".into(),
                },
                Account {
                    pk: "0x0000001111111112222222223333333333344444444445555555555666666666".into(),
                },
                Account {
                    pk: "0x1230000000000000000000000000000000000000000000000000000000000321".into(),
                },
                Account {
                    pk: "0x1111000000000000000000000000000000000000000000000000000000001111".into(),
                },
            ];

            let btc_accounts = std::iter::repeat(btc_accounts)
                .flatten()
                .take(account_count)
                .collect();

            let eth_accounts = vec![Account {
                pk: "0x1234567891011121123456789101112112345678".into(),
            }];

            let eth_accounts = std::iter::repeat(eth_accounts)
                .flatten()
                .take(account_count)
                .collect();

            let mut accounts = HashMap::new();
            accounts.insert(Network::Bitcoin, btc_accounts);
            accounts.insert(Network::Ethereum, eth_accounts);

            Self { devices, accounts }
        }
    }

    #[async_trait]
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

        async fn discover_accounts(&self, _device: &Device, network: Network) -> Vec<Account> {
            self.accounts
                .get(&network)
                .cloned()
                .into_iter()
                .flatten()
                .collect()
        }
    }
}
