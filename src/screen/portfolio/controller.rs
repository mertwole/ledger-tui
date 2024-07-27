use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode};

use super::Model;
use crate::{
    api::{coin_price::CoinPriceApiT, ledger::LedgerApiT},
    screen::{EventExt, OutgoingMessage, WindowName},
};

pub(super) fn process_input<L: LedgerApiT, C: CoinPriceApiT>(
    model: &mut Model<L, C>,
) -> Option<OutgoingMessage> {
    if !event::poll(Duration::ZERO).unwrap() {
        return None;
    }

    let event = event::read().unwrap();

    if let Some(state) = model.state.as_mut() {
        if let Some(accounts) = state.device_accounts.as_ref() {
            if event.is_key_pressed(KeyCode::Enter) {
                if let Some(selected_idx) = model.selected_account {
                    // TODO: Don't ignore other accounts - let user choose it on portfolio window,
                    let selected = accounts[selected_idx].clone();
                    state.selected_account = Some((selected.0, selected.1[0].clone()));

                    return Some(OutgoingMessage::SwitchWindow(WindowName::Asset));
                }
            }

            let accounts_len = accounts.len();
            process_table_navigation(model, &event, accounts_len);
        }
    }

    if event.is_key_pressed(KeyCode::Char('d')) {
        return Some(OutgoingMessage::SwitchWindow(WindowName::DeviceSelection));
    }

    if event.is_key_pressed(KeyCode::Char('q')) {
        return Some(OutgoingMessage::Exit);
    }

    None
}

fn process_table_navigation<L: LedgerApiT, C: CoinPriceApiT>(
    model: &mut Model<L, C>,
    event: &Event,
    accounts_len: usize,
) {
    if event.is_key_pressed(KeyCode::Down) {
        let selected = model
            .selected_account
            .as_mut()
            .map(|sel| {
                *sel += 1;
                if *sel >= accounts_len {
                    *sel = accounts_len - 1;
                }
            })
            .is_some();

        if !selected {
            model.selected_account = if accounts_len == 0 { None } else { Some(0) };
        }
    }

    if event.is_key_pressed(KeyCode::Up) {
        let selected = model
            .selected_account
            .as_mut()
            .map(|sel| {
                *sel = sel.saturating_sub(1);
            })
            .is_some();

        if !selected {
            model.selected_account = if accounts_len == 0 {
                None
            } else {
                Some(accounts_len - 1)
            };
        }
    }
}
