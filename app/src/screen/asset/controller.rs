use input_mapping_derive::InputMapping;
use ratatui::crossterm::event::{Event, KeyCode};

use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        ledger::LedgerApiT,
    },
    screen::{EventExt, OutgoingMessage, ScreenName},
};

use super::{Model, TimePeriod};

#[derive(InputMapping)]
pub enum InputEvent {
    #[key = 'q']
    #[description = "Quit application"]
    Quit,

    #[key = 'b']
    #[description = "Return one screen back"]
    Back,

    #[key = 's']
    #[description = "Open deposit screen"]
    OpenDepositScreen,

    SelectTimeInterval(SelectTimeInterval),
}

#[derive(InputMapping)]
pub enum SelectTimeInterval {
    #[key = 'd']
    #[description = "Select time interval - day"]
    Day,

    #[key = 'w']
    #[description = "Select time interval - week"]
    Week,

    #[key = 'm']
    #[description = "Select time interval - month"]
    Month,

    #[key = 'y']
    #[description = "Select time interval - year"]
    Year,

    #[key = 'a']
    #[description = "Select time interval - all time"]
    All,
}

pub(super) fn process_input<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    event: &Event,
    model: &mut Model<L, C, M>,
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

fn process_time_interval_selection<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    event: &Event,
    model: &mut Model<L, C, M>,
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
