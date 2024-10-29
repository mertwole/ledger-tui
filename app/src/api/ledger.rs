use std::{hash::Hash, str::FromStr};

use api_proc_macro::implement_cache;
use async_trait::async_trait;
use ledger_apdu::{APDUCommand, APDUErrorCode};
use ledger_transport_hid::{
    hidapi::{DeviceInfo as LedgerDeviceInfo, HidApi},
    TransportNativeHID,
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
pub struct Device {
    info: LedgerDeviceInfo,
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.info.path() == other.info.path()
            && self.info.serial_number() == other.info.serial_number()
    }
}

impl Eq for Device {}

impl Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.info.path().hash(state);
        self.info.serial_number().hash(state);
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
        let devices: Vec<_> = devices.map(|info| Device { info: info.clone() }).collect();

        log::info!("Discovered {} connected ledger devices", devices.len());

        devices
    }

    async fn get_device_info(&self, device: &Device) -> Option<DeviceInfo> {
        let model = device
            .info
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
        let transport = TransportNativeHID::open_device(&self.hid_api, &device.info).unwrap();

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
    use super::*;

    // TODO: Implement mock.
    pub struct LedgerApiMock {}

    impl LedgerApiMock {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[async_trait]
    impl LedgerApiT for LedgerApiMock {
        async fn discover_devices(&self) -> Vec<Device> {
            vec![]
        }

        async fn get_device_info(&self, _device: &Device) -> Option<DeviceInfo> {
            None
        }

        async fn discover_accounts(&self, _device: &Device, _network: Network) -> Vec<Account> {
            vec![]
        }
    }
}
