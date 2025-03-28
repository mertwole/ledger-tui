use std::str::FromStr;

use ledger_apdu::{APDUCommand, APDUErrorCode};
use ledger_transport_hid::{TransportNativeHID, hidapi::HidApi};

use crate::api::common_types::Account;

use super::Device;

pub async fn discover_accounts(device: &Device) -> Vec<Account> {
    log::info!("Discovering bitcoin accounts...");

    let device_info = device.get_info().expect("Expected non-mock device");
    let hid_api = HidApi::new().unwrap();
    let transport = TransportNativeHID::open_device(&hid_api, device_info).unwrap();

    #[allow(clippy::identity_op)]
    let data = &[
        // Display
        &[0u8][..],
        // Number of BIP 32 derivations to perform (max 8)
        &[5u8][..],
        // 1st derivation index (big endian)
        &((1u32 << 31) ^ 84u32).to_be_bytes()[..],
        // 2nd derivation index (big endian)
        &((1u32 << 31) ^ 0u32).to_be_bytes()[..],
        // 3rd derivation index (big endian)
        &((1u32 << 31) ^ 0u32).to_be_bytes()[..],
        // 4th derivation index (big endian)
        &0u32.to_be_bytes()[..],
        // 5th derivation index (big endian)
        &0u32.to_be_bytes()[..],
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

    log::info!(
        "Discovered bitcoin account with public key = {}",
        public_key
    );

    vec![Account { public_key }]
}

pub async fn sign_message(_message: Vec<u8>, _device: &Device) -> Vec<u8> {
    todo!()
}
