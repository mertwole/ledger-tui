use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    Frame,
};
use resources::Resources;

use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        ledger::LedgerApiT,
    },
    app::{ApiRegistry, StateRegistry},
};

pub mod asset;
mod common;
pub mod deposit;
pub mod device_selection;
pub mod portfolio;
pub mod resources;

pub enum Screen<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    Asset(asset::Model<L, C, M>),
    Deposit(deposit::Model<L, C, M>),
    DeviceSelection(device_selection::Model<L, C, M>),
    Portfolio(portfolio::Model<L, C, M>),
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Screen<L, C, M> {
    pub fn new(
        name: ScreenName,
        state_registry: StateRegistry,
        api_registry: ApiRegistry<L, C, M>,
    ) -> Self {
        match name {
            ScreenName::Asset => Self::Asset(asset::Model::construct(state_registry, api_registry)),
            ScreenName::Deposit => {
                Self::Deposit(deposit::Model::construct(state_registry, api_registry))
            }
            ScreenName::DeviceSelection => Self::DeviceSelection(
                device_selection::Model::construct(state_registry, api_registry),
            ),
            ScreenName::Portfolio => {
                Self::Portfolio(portfolio::Model::construct(state_registry, api_registry))
            }
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        match self {
            Self::Asset(screen) => screen.render(frame, resources),
            Self::Deposit(screen) => screen.render(frame, resources),
            Self::DeviceSelection(screen) => screen.render(frame, resources),
            Self::Portfolio(screen) => screen.render(frame, resources),
        }
    }

    pub fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        match self {
            Self::Asset(screen) => screen.tick(event),
            Self::Deposit(screen) => screen.tick(event),
            Self::DeviceSelection(screen) => screen.tick(event),
            Self::Portfolio(screen) => screen.tick(event),
        }
    }

    pub fn deconstruct(self) -> (StateRegistry, ApiRegistry<L, C, M>) {
        match self {
            Self::Asset(screen) => screen.deconstruct(),
            Self::Deposit(screen) => screen.deconstruct(),
            Self::DeviceSelection(screen) => screen.deconstruct(),
            Self::Portfolio(screen) => screen.deconstruct(),
        }
    }
}

trait ScreenT<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    fn construct(state: StateRegistry, api_registry: ApiRegistry<L, C, M>) -> Self;

    fn render(&self, frame: &mut Frame<'_>, resources: &Resources);
    fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage>;

    fn deconstruct(self) -> (StateRegistry, ApiRegistry<L, C, M>);
}

pub enum OutgoingMessage {
    SwitchScreen(ScreenName),
    Back,
    Exit,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScreenName {
    DeviceSelection,
    Portfolio,
    Asset,
    Deposit,
}

trait EventExt {
    fn is_key_pressed(&self, code: KeyCode) -> bool;
}

impl EventExt for Event {
    fn is_key_pressed(&self, code: KeyCode) -> bool {
        let pressed_code = code;

        matches!(
            self,
            &Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code,
                ..
            }) if code == pressed_code
        )
    }
}
