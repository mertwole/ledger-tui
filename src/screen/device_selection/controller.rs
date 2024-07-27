use std::time::Duration;

use ratatui::crossterm::event::{self, KeyCode};

use crate::{
    api::ledger::LedgerApiT,
    screen::{EventExt, OutgoingMessage, ScreenName},
};

use super::Model;

pub(super) fn process_input<L: LedgerApiT>(model: &mut Model<L>) -> Option<OutgoingMessage> {
    if !event::poll(Duration::ZERO).unwrap() {
        return None;
    }

    let event = event::read().unwrap();

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
            model
                .state
                .as_mut()
                .expect("Construct should be called at the start of window lifetime")
                .active_device = Some((device, info));
            // TODO: Add mechanism to return one window back.
            return Some(OutgoingMessage::SwitchScreen(ScreenName::Portfolio));
        }
    }

    if event.is_key_pressed(KeyCode::Char('q')) {
        return Some(OutgoingMessage::Exit);
    }

    None
}
