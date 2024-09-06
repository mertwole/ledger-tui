use input_mapping_common::InputMappingT;
use input_mapping_derive::InputMapping;
use ratatui::crossterm::event::Event;

use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        ledger::LedgerApiT,
    },
    screen::{OutgoingMessage, ScreenName},
};

use super::{Model, TimePeriod};

#[derive(InputMapping)]
pub enum InputEvent {
    #[key = 'q']
    #[description = "Quit application"]
    Quit,

    #[key = 'h']
    #[description = "Open/close navigation help"]
    NavigationHelp,

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
    let event = InputEvent::map_event(event.clone())?;

    match event {
        InputEvent::Quit => Some(OutgoingMessage::Exit),
        InputEvent::NavigationHelp => {
            model.show_navigation_help ^= true;
            None
        }
        InputEvent::Back => Some(OutgoingMessage::Back),
        InputEvent::OpenDepositScreen => Some(OutgoingMessage::SwitchScreen(ScreenName::Deposit)),
        InputEvent::SelectTimeInterval(event) => {
            model.selected_time_period = match event {
                SelectTimeInterval::Day => TimePeriod::Day,
                SelectTimeInterval::Week => TimePeriod::Week,
                SelectTimeInterval::Month => TimePeriod::Month,
                SelectTimeInterval::Year => TimePeriod::Year,
                SelectTimeInterval::All => TimePeriod::All,
            };

            None
        }
    }
}
