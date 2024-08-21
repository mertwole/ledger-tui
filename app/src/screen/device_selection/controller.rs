use ratatui::crossterm::event::{Event, KeyCode};

use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        ledger::LedgerApiT,
    },
    screen::{EventExt, OutgoingMessage, ScreenName},
};

use super::Model;

pub(super) fn process_input<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &mut Model<L, C, M>,
    event: &Event,
) -> Option<OutgoingMessage> {
    if event.is_key_pressed(KeyCode::Down) && !model.devices.is_empty() {
        if let Some(selected) = model.selected_device.as_mut() {
            *selected = (model.devices.len() - 1).min(*selected + 1);
        } else {
            model.selected_device = Some(0);
        }
    }

    if event.is_key_pressed(KeyCode::Up) && !model.devices.is_empty() {
        if let Some(selected) = model.selected_device.as_mut() {
            *selected = if *selected == 0 { 0 } else { *selected - 1 };
        } else {
            model.selected_device = Some(model.devices.len() - 1);
        }
    }

    if event.is_key_pressed(KeyCode::Enter) {
        if let Some(device_idx) = model.selected_device {
            let (device, info) = model.devices[device_idx].clone();
            model.state.active_device = Some((device, info));

            return Some(OutgoingMessage::Back);
        }
    }

    if event.is_key_pressed(KeyCode::Char('q')) {
        return Some(OutgoingMessage::Exit);
    }

    None
}
