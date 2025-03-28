use std::{hash::Hash, time::Duration};

use async_trait::async_trait;
use ledger_apdu::{APDUCommand, APDUErrorCode};
use ledger_transport_hid::{
    TransportNativeHID,
    hidapi::{DeviceInfo as LedgerDeviceInfo, HidApi},
};

use super::common_types::{Account, Network};

mod bitcoin_app;
mod ethereum_app;

const DELAY_BEFORE_ACCOUNTS_DISCOVERY: Duration = Duration::from_secs(1);

#[async_trait]
pub trait LedgerApiT: Send + Sync + 'static {
    async fn discover_devices(&self) -> Vec<Device>;

    async fn get_device_info(&self, device: &Device) -> Option<DeviceInfo>;

    async fn open_app(&self, device: &Device, network: Network) -> ();

    // TODO: Return stream of accounts?
    async fn discover_accounts(&self, device: &Device, network: Network) -> Vec<Account>;

    #[allow(dead_code)] // TODO: Remove.
    async fn sign_message(&self, message: Vec<u8>, device: &Device, network: Network) -> Vec<u8>;
}

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

pub struct LedgerApi {}

impl LedgerApi {
    pub async fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl LedgerApiT for LedgerApi {
    async fn discover_devices(&self) -> Vec<Device> {
        log::info!("Discovering connected ledger devices...");

        let hid_api = HidApi::new().unwrap();
        let devices = TransportNativeHID::list_ledgers(&hid_api);
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

    async fn open_app(&self, device: &Device, network: Network) -> () {
        let device_info = device.get_info().expect("Expected non-mock device");
        let hid_api = HidApi::new().unwrap();
        let transport = TransportNativeHID::open_device(&hid_api, device_info).unwrap();

        let data = match network {
            Network::Bitcoin => "Bitcoin",
            Network::Ethereum => "Ethereum",
        }
        .as_bytes();

        let command = APDUCommand {
            cla: 0xE0,
            ins: 0xD8,
            p1: 0x00,
            p2: 0x00,
            data,
        };

        let response = transport.exchange(&command).unwrap();

        match response.error_code() {
            Err(e) => {
                log::error!("Error received from ledegr device: {:#x}", e);
            }
            Ok(APDUErrorCode::NoError) => {}
            Ok(e) => {
                log::error!("Error received from ledegr device: {}", e);
            }
        }

        ()
    }

    async fn discover_accounts(&self, device: &Device, network: Network) -> Vec<Account> {
        // TODO: It's a workaround of a problem that ledger disconnects after `open_app` request
        // and so maintainionhg connection to it is impossible, so we try to reconnect to it
        // after some delay.
        tokio::time::sleep(DELAY_BEFORE_ACCOUNTS_DISCOVERY).await;

        match network {
            Network::Bitcoin => bitcoin_app::discover_accounts(device).await,
            Network::Ethereum => ethereum_app::discover_accounts(device).await,
        }
    }

    async fn sign_message(&self, message: Vec<u8>, device: &Device, network: Network) -> Vec<u8> {
        // TODO: It's a workaround of a problem that ledger disconnects after `open_app` request
        // and so maintainionhg connection to it is impossible, so we try to reconnect to it
        // after some delay.
        tokio::time::sleep(DELAY_BEFORE_ACCOUNTS_DISCOVERY).await;

        match network {
            Network::Bitcoin => bitcoin_app::sign_message(message, device).await,
            Network::Ethereum => ethereum_app::sign_message(message, device).await,
        }
    }
}

pub mod mock {
    use std::{collections::HashMap, sync::Mutex};

    use super::*;

    pub struct LedgerApiMock {
        devices: Vec<Device>,
        accounts: HashMap<Network, Vec<Account>>,
        open_app: Mutex<Option<Network>>,
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

            let eth_accounts = repeat_accounts(vec![
                "0x1234567891011121123456789101112112345678",
                "0x1111111111111111111111111111111111111111",
                "0x3333333333333333333333333333333333333333",
                "0x5555555555555555555555555555555555555555",
            ]);
            accounts.insert(Network::Ethereum, eth_accounts);

            Self {
                devices: (0..device_count).map(Device::new_mock).collect(),
                accounts,
                open_app: Mutex::new(None),
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

        async fn open_app(&self, _device: &Device, network: Network) {
            *self.open_app.lock().unwrap() = Some(network);
        }

        async fn discover_accounts(&self, _device: &Device, network: Network) -> Vec<Account> {
            assert_eq!(*self.open_app.lock().unwrap(), Some(network));

            self.accounts
                .get(&network)
                .cloned()
                .into_iter()
                .flatten()
                .collect()
        }

        async fn sign_message(
            &self,
            _message: Vec<u8>,
            _device: &Device,
            _network: Network,
        ) -> Vec<u8> {
            todo!()
        }
    }
}
