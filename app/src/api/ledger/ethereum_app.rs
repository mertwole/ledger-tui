use ledger_apdu::{APDUCommand, APDUErrorCode};
use ledger_transport_hid::{TransportNativeHID, hidapi::HidApi};

use crate::api::common_types::Account;

use super::Device;

pub async fn discover_accounts(device: &Device) -> Vec<Account> {
    log::info!("Discovering ethereum accounts");

    let device_info = device.get_info().expect("Expected non-mock device");
    let hid_api = HidApi::new().unwrap();
    let transport = TransportNativeHID::open_device(&hid_api, device_info).unwrap();

    let data = &[
        &encode_default_derivation_path()[..],
        //Optional - 8 bytes for chain id.
    ]
    .concat()[..];

    let command = APDUCommand {
        cla: 0xE0,
        ins: 0x02,
        p1: 0x00, // 0x00 - return address; 0x01 - display address and return.
        p2: 0x00, // 0x00 - do not return the chain code; 0x01 - return the chain code.
        data,
    };

    let Ok(response) = send_command(&command, &transport) else {
        return vec![];
    };

    let public_key_length = response[0] as usize;
    let _public_key = &response[1..1 + public_key_length];

    let ethereum_address_length = response[1 + public_key_length] as usize;
    let ethereum_address =
        &response[1 + public_key_length + 1..1 + public_key_length + 1 + ethereum_address_length];

    let public_key = String::from_utf8(ethereum_address.to_vec()).unwrap();
    let public_key = ["0x", &public_key].concat();

    log::info!(
        "Discovered ethereum account with public key = {}",
        public_key,
    );

    vec![Account { public_key }]
}

const MESSAGE_CHUNK_SIZE: usize = 255;

pub async fn sign_message(message: Vec<u8>, device: &Device) -> Vec<u8> {
    log::info!("Signing ethereum message 0x{}", hex::encode(&message));

    let device_info = device.get_info().expect("Expected non-mock device");
    let hid_api = HidApi::new().unwrap();
    let transport = TransportNativeHID::open_device(&hid_api, device_info).unwrap();

    let chunks: Vec<_> = message.chunks(MESSAGE_CHUNK_SIZE).collect();
    let first_chunk = chunks[0];
    let remaining_chunks = if chunks.len() == 1 { &[] } else { &chunks[1..] };

    let data = &[
        &encode_default_derivation_path()[..],
        //Optional - 8 bytes for chain id.
    ]
    .concat()[..];

    let p2 = if remaining_chunks.is_empty() {
        0x00
    } else {
        0x01
    };

    let command = APDUCommand {
        cla: 0xE0,
        ins: 0x04,
        p1: 0x00, // 0x00 - first transaction data block; 0x80 - subsequent transaction data block.
        p2,       // 0x00 - process & start flow; 0x01 - store only; 0x02 - start flow.
        data,
    };

    let Ok(response) = send_command(&command, &transport) else {
        return vec![];
    };

    if remaining_chunks.is_empty() {
        let v = response[0];
        let r = &response[1..33];
        let s = &response[33..65];

        log::info!(
            "Ethereum message signature: v={:#04x} r=0x{} s=0x{}",
            v,
            hex::encode(r),
            hex::encode(s)
        );

        return response;
    }

    for chunk in &remaining_chunks[..remaining_chunks.len() - 1] {
        let command = APDUCommand {
            cla: 0xE0,
            ins: 0x04,
            p1: 0x80,
            p2: 0x01,
            data: *chunk,
        };

        let Ok(_) = send_command(&command, &transport) else {
            return vec![];
        };
    }

    let last_chunk = remaining_chunks
        .last()
        .expect("Checked that remaining_chunks is not empty");

    let command = APDUCommand {
        cla: 0xE0,
        ins: 0x04,
        p1: 0x80,
        p2: 0x00,
        data: *last_chunk,
    };

    let Ok(response) = send_command(&command, &transport) else {
        return vec![];
    };

    let v = response[0];
    let r = &response[1..33];
    let s = &response[33..65];

    log::info!(
        "Ethereum message signature: v={:#04x} r=0x{} s=0x{}",
        v,
        hex::encode(r),
        hex::encode(s)
    );

    response
}

type CommandResult = Result<Vec<u8>, ()>;

fn send_command(command: &APDUCommand<&[u8]>, transport: &TransportNativeHID) -> CommandResult {
    let response = transport.exchange(&command).unwrap();

    match response.error_code() {
        Err(error_code) => {
            log::error!("Error returned from ledger device: {:#010x}", error_code);
            Err(())
        }
        Ok(APDUErrorCode::NoError) => Ok(response.data().to_vec()),
        Ok(error) => {
            log::error!("Error returned from ledger device: {}", error);
            Err(())
        }
    }
}

fn encode_default_derivation_path() -> Vec<u8> {
    #[allow(clippy::identity_op)]
    [
        // Number of BIP 32 derivations to perform (max 10)
        &[4u8][..],
        // 1st derivation index (big endian)
        &((1u32 << 31) ^ 44u32).to_be_bytes()[..],
        // 2nd derivation index (big endian)
        &((1u32 << 31) ^ 60u32).to_be_bytes()[..],
        // 3rd derivation index (big endian)
        &((1u32 << 31) ^ 0u32).to_be_bytes()[..],
        // 4th derivation index (big endian)
        &0u32.to_be_bytes()[..],
    ]
    .concat()
}
