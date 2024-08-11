use ratatui::crossterm::event::{Event, KeyCode};

use crate::{
    api::{blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT},
    screen::{EventExt, OutgoingMessage, ScreenName},
};

use super::{Model, TimePeriod};

pub(super) fn process_input<C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    event: &Event,
    model: &mut Model<C, M>,
) -> Option<OutgoingMessage> {
    if event.is_key_pressed(KeyCode::Char('q')) {
        return Some(OutgoingMessage::Exit);
    }

    if event.is_key_pressed(KeyCode::Char('b')) {
        return Some(OutgoingMessage::Back);
    }

    if event.is_key_pressed(KeyCode::Char('s')) {
        return Some(OutgoingMessage::SwitchScreen(ScreenName::Deposit));
    }

    process_time_interval_selection(event, model);

    None
}

fn process_time_interval_selection<C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    event: &Event,
    model: &mut Model<C, M>,
) {
    match () {
        () if event.is_key_pressed(KeyCode::Char('d')) => {
            model.selected_time_period = TimePeriod::Day;
        }
        () if event.is_key_pressed(KeyCode::Char('w')) => {
            model.selected_time_period = TimePeriod::Week;
        }
        () if event.is_key_pressed(KeyCode::Char('m')) => {
            model.selected_time_period = TimePeriod::Month;
        }
        () if event.is_key_pressed(KeyCode::Char('y')) => {
            model.selected_time_period = TimePeriod::Year;
        }
        () if event.is_key_pressed(KeyCode::Char('a')) => {
            model.selected_time_period = TimePeriod::All;
        }
        _ => {}
    };
}
