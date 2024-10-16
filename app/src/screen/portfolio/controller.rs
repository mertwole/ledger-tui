use std::num::NonZeroUsize;

use input_mapping_common::InputMappingT;
use input_mapping_derive::InputMapping;
use ratatui::crossterm::event::{Event, KeyCode};

use super::Model;
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        ledger::LedgerApiT,
    },
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

    #[key = "KeyCode::Down"]
    #[description = "Navigate down in list"]
    Down,

    #[key = "KeyCode::Up"]
    #[description = "Navigate up in list"]
    Up,

    #[key = "KeyCode::Enter"]
    #[description = "Select device"]
    Select,
}

pub(super) fn process_input<
    L: LedgerApiT,
    C: CoinPriceApiT + Clone + 'static,
    M: BlockchainMonitoringApiT,
>(
    event: &Event,
    model: &mut Model<L, C, M>,
) -> Option<OutgoingMessage> {
    let event = InputEvent::map_event(event.clone())?;

    match event {
        InputEvent::Quit => return Some(OutgoingMessage::Exit),
        InputEvent::NavigationHelp => {
            model.show_navigation_help ^= true;
            return None;
        }
        InputEvent::OpenDeviceSelection => {
            return Some(OutgoingMessage::SwitchScreen(ScreenName::DeviceSelection))
        }
        _ => {}
    };

    if let Some(accounts) = model.state.device_accounts.as_ref() {
        if matches!(event, InputEvent::Select) {
            if let Some((selected_network_idx, selected_account_idx)) = model.selected_account {
                let (selected_network, accounts) = &accounts[selected_network_idx];
                let selected_account = accounts[selected_account_idx].clone();

                model.state.selected_account = Some((*selected_network, selected_account));

                return Some(OutgoingMessage::SwitchScreen(ScreenName::Asset));
            }
        }

        let accounts_per_network: Vec<_> = accounts
            .iter()
            .map(|(_, accounts)| {
                NonZeroUsize::new(accounts.len()).expect("No accounts for provided network found")
            })
            .collect();

        process_table_navigation(model, &event, &accounts_per_network);
    }

    None
}

fn process_table_navigation<
    L: LedgerApiT,
    C: CoinPriceApiT + Clone + 'static,
    M: BlockchainMonitoringApiT,
>(
    model: &mut Model<L, C, M>,
    event: &InputEvent,
    accounts_per_network: &[NonZeroUsize],
) {
    if matches!(event, InputEvent::Down) {
        if let Some((selected_network, selected_account)) = model.selected_account {
            if selected_account + 1 >= accounts_per_network[selected_network].into() {
                if selected_network >= accounts_per_network.len() - 1 {
                    let last_network_accounts: usize =
                        (*accounts_per_network.last().unwrap()).into();

                    model.selected_account =
                        Some((accounts_per_network.len() - 1, last_network_accounts - 1));
                } else {
                    model.selected_account = Some((selected_network + 1, 0));
                }
            } else {
                model.selected_account = Some((selected_network, selected_account + 1));
            }
        } else {
            model.selected_account = if accounts_per_network.is_empty() {
                None
            } else {
                Some((0, 0))
            };
        }
    }

    if matches!(event, InputEvent::Up) {
        if let Some((selected_network, selected_account)) = model.selected_account {
            if selected_account == 0 {
                if selected_network == 0 {
                    model.selected_account = Some((0, 0));
                } else {
                    let accounts: usize = accounts_per_network[selected_network - 1].into();
                    model.selected_account = Some((selected_network - 1, accounts - 1));
                }
            } else {
                model.selected_account = Some((selected_network, selected_account - 1));
            }
        } else {
            model.selected_account = if accounts_per_network.is_empty() {
                None
            } else {
                let network = accounts_per_network.len() - 1;
                let account: usize = accounts_per_network[network].into();
                let account = account - 1;
                Some((network, account))
            };
        }
    }
}
