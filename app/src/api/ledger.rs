use std::{hash::Hash, str::FromStr};

use api_proc_macro::implement_cache;
use async_trait::async_trait;
use ledger_apdu::{APDUCommand, APDUErrorCode};
use ledger_transport_hid::{
    TransportNativeHID,
    hidapi::{DeviceInfo as LedgerDeviceInfo, HidApi},
};

use super::common_types::{Account, Network};

implement_cache!(
    #[async_trait]
    pub trait LedgerApiT: Send + Sync + 'static {
        async fn discover_devices(&self) -> Vec<Device>;

        async fn get_device_info(&self, device: &Device) -> Option<DeviceInfo>;

        // TODO: Return stream of accounts?
        async fn discover_accounts(&self, device: &Device, network: Network) -> Vec<Account>;
    }
);

#[derive(Clone, Debug)]
pub struct Device(DeviceInner);

impl Device {
    fn new(info: LedgerDeviceInfo) -> Self {
        Self(DeviceInner::Device(info))
    }

    fn new_mock(id: usize) -> Self {
        Self(DeviceInner::Mock(id))
    }

    fn get_info(&self) -> Option<&LedgerDeviceInfo> {
        match &self.0 {
            DeviceInner::Mock(_) => None,
            DeviceInner::Device(info) => Some(info),
        }
    }

    fn get_mock_id(&self) -> Option<usize> {
        match &self.0 {
            DeviceInner::Mock(id) => Some(*id),
            DeviceInner::Device(_) => None,
        }
    }
}

#[derive(Clone, Debug)]
enum DeviceInner {
    Mock(usize),
    Device(LedgerDeviceInfo),
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (DeviceInner::Mock(self_id), DeviceInner::Mock(other_id)) => self_id == other_id,
            (DeviceInner::Device(self_info), DeviceInner::Device(other_info)) => {
                self_info.path() == other_info.path()
                    && self_info.serial_number() == other_info.serial_number()
            }
            _ => false,
        }
    }
}

impl Eq for Device {}

impl Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.0 {
            DeviceInner::Mock(id) => id.hash(state),
            DeviceInner::Device(info) => {
                info.path().hash(state);
                info.serial_number().hash(state);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub model: String,
}

pub struct LedgerApi {
    hid_api: HidApi,
}

impl LedgerApi {
    pub async fn new() -> Self {
        Self {
            hid_api: HidApi::new().unwrap(),
        }
    }
}

#[async_trait]
impl LedgerApiT for LedgerApi {
    async fn discover_devices(&self) -> Vec<Device> {
        log::info!("Discovering connected ledger devices...");

        let devices = TransportNativeHID::list_ledgers(&self.hid_api);
        let devices: Vec<_> = devices.cloned().map(Device::new).collect();

        log::info!("Discovered {} connected ledger devices", devices.len());

        devices
    }

    async fn get_device_info(&self, device: &Device) -> Option<DeviceInfo> {
        let model = device
            .get_info()
            .expect("Expected non-mock device")
            .product_string()
            .map(|s| s.to_string())
            .unwrap_or_default();

        Some(DeviceInfo { model })
    }

    async fn discover_accounts(&self, device: &Device, network: Network) -> Vec<Account> {
        match network {
            Network::Bitcoin => self.discover_bitcoin_accounts(device).await,
            Network::Ethereum => self.discover_ethereum_accounts(device).await,
        }
    }
}

impl LedgerApi {
    // TODO: It's just a showcase of communicating with bitcoin app.
    async fn discover_bitcoin_accounts(&self, device: &Device) -> Vec<Account> {
        let device_info = device.get_info().expect("Expected non-mock device");
        let transport = TransportNativeHID::open_device(&self.hid_api, device_info).unwrap();

        #[allow(clippy::identity_op)]
        let data = &[
            &[0u8][..],                                // Display
            &[5u8][..], // Number of BIP 32 derivations to perform (max 8)
            &((1u32 << 31) ^ 84u32).to_be_bytes()[..], // 1st derivation index (big endian)
            &((1u32 << 31) ^ 0u32).to_be_bytes()[..], // 2nd derivation index (big endian)
            &((1u32 << 31) ^ 0u32).to_be_bytes()[..], // 3rd derivation index (big endian)
            &0u32.to_be_bytes()[..], // 4th derivation index (big endian)
            &0u32.to_be_bytes()[..], // 5th derivation index (big endian)
        ]
        .concat()[..];

        let command = APDUCommand {
            cla: 0xE1,
            ins: 0x00,
            p1: 0x00,
            p2: 0x00,
            data,
        };

        let response = transport.exchange(&command).unwrap();

        match response.error_code() {
            Err(_) => return vec![],
            Ok(APDUErrorCode::NoError) => {}
            Ok(_) => return vec![],
        }

        let response = String::from_utf8(response.data().to_vec()).unwrap();
        let xpub = bitcoin::bip32::ExtendedPubKey::from_str(&response).unwrap();

        let public_key = xpub.public_key.to_string();

        vec![Account { public_key }]
    }

    async fn discover_ethereum_accounts(&self, _device: &Device) -> Vec<Account> {
        vec![]
    }
}

pub mod mock {
    use std::collections::HashMap;

    use super::*;

    pub struct LedgerApiMock {
        devices: Vec<Device>,
        accounts: HashMap<Network, Vec<Account>>,
    }

    impl LedgerApiMock {
        pub fn new(device_count: usize, account_count: usize) -> Self {
            let repeat_accounts = |pattern: Vec<&str>| -> Vec<Account> {
                std::iter::repeat(pattern)
                    .flatten()
                    .take(account_count)
                    .map(|acc| Account {
                        public_key: acc.into(),
                    })
                    .collect()
            };

            let mut accounts = HashMap::new();

            let btc_accounts = repeat_accounts(vec![
                "0x0123456789012345678901234567890101234567890123456789012345678901",
                "0x9876543210987654321098765432109876543210987654321098765432109876",
                "0x0000001111111112222222223333333333344444444445555555555666666666",
                "0x1230000000000000000000000000000000000000000000000000000000000321",
                "0x1111000000000000000000000000000000000000000000000000000000001111",
            ]);
            accounts.insert(Network::Bitcoin, btc_accounts);

            let eth_accounts = repeat_accounts(vec!["0x1234567891011121123456789101112112345678"]);
            accounts.insert(Network::Ethereum, eth_accounts);

            Self {
                devices: (0..device_count).map(Device::new_mock).collect(),
                accounts,
            }
        }
    }

    #[async_trait]
    impl LedgerApiT for LedgerApiMock {
        async fn discover_devices(&self) -> Vec<Device> {
            self.devices.clone()
        }

        async fn get_device_info(&self, device: &Device) -> Option<DeviceInfo> {
            let id = device.get_mock_id().expect("Expected mock device");

            let info = match id % 3 {
                0 => DeviceInfo {
                    model: "Nano S".to_string(),
                },
                1 => DeviceInfo {
                    model: "Nano S+".to_string(),
                },
                2 => DeviceInfo {
                    model: "Nano X".to_string(),
                },
                _ => unreachable!(),
            };

            Some(info)
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
