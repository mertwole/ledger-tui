use input_mapping_common::InputMappingT;
use input_mapping_derive::InputMapping;
use ratatui::crossterm::event::{Event, KeyCode};

use crate::{api::ledger::LedgerApiT, screen::OutgoingMessage};

use super::Model;

#[derive(InputMapping)]
pub enum InputEvent {
    #[key = 'q']
    #[description = "Quit application"]
    Quit,

    #[key = 'h']
    #[description = "Open/close navigation help"]
    NavigationHelp,

    #[key = "KeyCode::Down"]
    #[description = "Navigate down in list"]
    Down,

    #[key = "KeyCode::Up"]
    #[description = "Navigate up in list"]
    Up,

    #[key = "KeyCode::Enter"]
    #[description = "Select device"]
    Select,

    #[key = 'r']
    #[description = "Refresh device list"]
    Refresh,
}

pub(super) fn process_input<L: LedgerApiT>(
    event: &Event,
    model: &mut Model<L>,
) -> Option<OutgoingMessage> {
    let event = InputEvent::map_event(event.clone())?;

    match event {
        InputEvent::Quit => Some(OutgoingMessage::Exit),
        InputEvent::NavigationHelp => {
            model.show_navigation_help ^= true;
            None
        }
        InputEvent::Down => {
            if !model.devices.is_empty() {
                if let Some(selected) = model.selected_device.as_mut() {
                    *selected = (model.devices.len() - 1).min(*selected + 1);
                } else {
                    model.selected_device = Some(0);
                }
            }

            None
        }
        InputEvent::Up => {
            if !model.devices.is_empty() {
                if let Some(selected) = model.selected_device.as_mut() {
                    *selected = if *selected == 0 { 0 } else { *selected - 1 };
                } else {
                    model.selected_device = Some(model.devices.len() - 1);
                }
            }

            None
        }
        InputEvent::Select => {
            if let Some(device_idx) = model.selected_device {
                model.state.active_device = Some(model.devices[device_idx].clone());

                Some(OutgoingMessage::Back)
            } else {
                None
            }
        }
        InputEvent::Refresh => {
            model.refresh_device_list();
            None
        }
    }
}
