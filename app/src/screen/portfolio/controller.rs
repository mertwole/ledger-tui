use input_mapping_common::InputMappingT;
use input_mapping_derive::InputMapping;
use ratatui::crossterm::event::{Event, KeyCode};

use super::{Model, NetworkIdx};
use crate::{
    api::{blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT},
    screen::{OutgoingMessage, ScreenName},
};

#[derive(InputMapping)]
pub enum InputEvent {
    #[key = 'q']
    #[description = "Quit application"]
    Quit,

    #[key = 'h']
    #[description = "Open/close navigation help"]
    NavigationHelp,

    #[key = 'd']
    #[description = "Open device selection screen"]
    OpenDeviceSelection,

    #[key = 'a']
    #[description = "Open account discovery screen"]
    OpenAccountDiscovery,

    #[key = "KeyCode::Down"]
    #[description = "Navigate down in list"]
    Down,

    #[key = "KeyCode::Up"]
    #[description = "Navigate up in list"]
    Up,

    #[key = "KeyCode::Enter"]
    #[description = "Select account"]
    Select,
}

pub(super) fn process_input<C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    event: &Event,
    model: &mut Model<C, M>,
) -> Option<OutgoingMessage> {
    let event = InputEvent::map_event(event.clone())?;

    match event {
        InputEvent::Quit => return Some(OutgoingMessage::Exit),
        InputEvent::NavigationHelp => {
            model.show_navigation_help ^= true;
            return None;
        }
        InputEvent::OpenDeviceSelection => {
            return Some(OutgoingMessage::SwitchScreen(ScreenName::DeviceSelection));
        }
        InputEvent::OpenAccountDiscovery => {
            return Some(OutgoingMessage::SwitchScreen(ScreenName::AccountDiscovery));
        }
        _ => {}
    };

    let accounts = model
        .state
        .device_accounts
        .as_ref()
        .expect("TODO: Enforce this rule at app level?");

    if matches!(event, InputEvent::Select) {
        match (model.selected_network, model.selected_account) {
            (Some(network_idx), Some(account_idx)) => {
                let (selected_network, accounts) = &accounts[network_idx];
                let selected_account = accounts[account_idx].clone();

                model.state.selected_account = Some((*selected_network, selected_account));

                return Some(OutgoingMessage::SwitchScreen(ScreenName::Asset));
            }
            _ => {}
        }
    }

    let accounts_per_network: Vec<_> = accounts
        .iter()
        .map(|(_, accounts)| accounts.len())
        .collect();

    process_table_navigation(
        &mut model.selected_network,
        &mut model.selected_account,
        &event,
        &accounts_per_network,
    );

    None
}

fn process_table_navigation(
    selected_network: &mut Option<NetworkIdx>,
    selected_account: &mut Option<NetworkIdx>,
    event: &InputEvent,
    accounts_per_network: &[usize],
) {
    // TODO: Refactor these if-else.
    if matches!(event, InputEvent::Down) {
        if let Some(selected_network_idx) = selected_network {
            if let Some(selected_account_idx) = selected_account {
                if *selected_account_idx + 1 < accounts_per_network[*selected_network_idx] {
                    *selected_account_idx += 1;
                } else {
                    if *selected_network_idx + 1 < accounts_per_network.len() {
                        *selected_network_idx += 1;
                        *selected_account = None;
                    }
                }
            } else {
                if accounts_per_network[*selected_network_idx] == 0 {
                    if *selected_network_idx + 1 < accounts_per_network.len() {
                        *selected_network_idx += 1;
                    }
                } else {
                    *selected_account = Some(0);
                }
            }
        } else {
            if !accounts_per_network.is_empty() {
                *selected_network = Some(0);
            }
        }
    }

    if matches!(event, InputEvent::Up) {
        if let Some(selected_network_idx) = selected_network {
            if let Some(selected_account_idx) = selected_account {
                if *selected_account_idx > 0 {
                    *selected_account_idx -= 1;
                } else {
                    *selected_account = None;
                }
            } else {
                if *selected_network_idx != 0 {
                    *selected_network_idx -= 1;
                    let accounts_len = accounts_per_network[*selected_network_idx];
                    if accounts_len != 0 {
                        *selected_account = Some(accounts_len - 1);
                    }
                }
            }
        } else {
            if !accounts_per_network.is_empty() {
                *selected_network = Some(accounts_per_network.len() - 1);
                let last_accounts_len = *accounts_per_network
                    .last()
                    .expect("accounts_per_network checked to be non-empty");
                if last_accounts_len != 0 {
                    *selected_account = Some(last_accounts_len - 1);
                }
            }
        }
    }
}
