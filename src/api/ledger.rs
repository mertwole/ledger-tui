use futures::executor::block_on;
use ledger_lib::{
    info::Model, Device as LedgerDevice, Filters, LedgerInfo, LedgerProvider, Transport,
    DEFAULT_TIMEOUT,
};
use std::cell::RefCell;

pub trait LedgerApiT {
    async fn discover_devices(&self) -> Vec<Device>;

    async fn get_device_info(&self, device: &Device) -> Option<DeviceInfo>;

    async fn discover_accounts(
        &self,
        device: &Device,
        network: Network,
    ) -> impl Iterator<Item = Account>;
}

#[derive(Clone, Debug)]
pub struct Device {
    info: LedgerInfo,
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub model: String,
    pub se_version: String,
    pub mcu_version: String,
}

// TODO: Move it level up as it'll be shared between ledger and market APIs.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Network {
    Bitcoin,
    Ethereum,
}

pub struct NetworkInfo {
    pub name: String,
    pub symbol: String,
}

impl Network {
    pub fn get_info(&self) -> NetworkInfo {
        match self {
            Self::Bitcoin => NetworkInfo {
                name: "Bitcoin".into(),
                symbol: "BTC".into(),
            },
            Self::Ethereum => NetworkInfo {
                name: "Ethereum".into(),
                symbol: "ETH".into(),
            },
        }
    }
}

#[derive(Clone)]
pub struct Account {
    pub(self) pk: String,
}

impl Account {
    pub fn get_info(&self) -> AccountInfo {
        AccountInfo {
            pk: self.pk.clone(),
        }
    }
}

pub struct AccountInfo {
    #[allow(dead_code)]
    pub pk: String,
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

    async fn discover_accounts(
        &self,
        _device: &Device,
        _network: Network,
    ) -> impl Iterator<Item = Account> {
        todo!();

        #[allow(unreachable_code)]
        None.into_iter()
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

        async fn discover_accounts(
            &self,
            _device: &Device,
            network: Network,
        ) -> impl Iterator<Item = Account> {
            self.accounts.get(&network).cloned().into_iter().flatten()
        }
    }
}
