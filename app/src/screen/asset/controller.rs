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
    Quit,
    #[key = 'b']
    Back,
    #[key = 's']
    #[description = "Open deposit screen"]
    OpenDepositScreen,
    SelectTimeInterval(SelectTimeIntervalEvent),
}

#[derive(InputMapping)]
pub enum SelectTimeIntervalEvent {
    #[key = 'd']
    Day,
    #[key = 'w']
    Week,
    #[key = 'm']
    Month,
    #[key = 'y']
    Year,
    #[key = 'a']
    All,
}

impl InputEvent {
    pub fn from_event(event: &Event) -> Option<Self> {
        match () {
            () if event.is_key_pressed(KeyCode::Char('q')) => Some(Self::Quit),
            () if event.is_key_pressed(KeyCode::Char('b')) => Some(Self::Back),
            () if event.is_key_pressed(KeyCode::Char('s')) => Some(Self::OpenDepositScreen),
            _ => SelectTimeIntervalEvent::from_event(event).map(|e| Self::SelectTimeInterval(e)),
        }
    }
}

impl SelectTimeIntervalEvent {
    fn from_event(event: &Event) -> Option<Self> {
        match () {
            () if event.is_key_pressed(KeyCode::Char('d')) => Some(Self::Day),
            () if event.is_key_pressed(KeyCode::Char('w')) => Some(Self::Week),
            () if event.is_key_pressed(KeyCode::Char('m')) => Some(Self::Month),
            () if event.is_key_pressed(KeyCode::Char('y')) => Some(Self::Year),
            () if event.is_key_pressed(KeyCode::Char('a')) => Some(Self::All),
            _ => None,
        }
    }
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
