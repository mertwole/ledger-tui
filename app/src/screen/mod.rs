use ratatui::{Frame, crossterm::event::Event};
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

#[allow(clippy::large_enum_variant)]
pub struct Screen<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    remaining_apis: ApiRegistry<L, C, M>,
    model: ScreenModel<L, C, M>,
}

enum ScreenModel<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> {
    Asset(asset::Model<C, M>),
    Deposit(deposit::Model),
    DeviceSelection(device_selection::Model<L>),
    Portfolio(portfolio::Model<L, C, M>),
}

impl<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT> Screen<L, C, M> {
    pub fn new(
        name: ScreenName,
        state_registry: StateRegistry,
        api_registry: ApiRegistry<L, C, M>,
    ) -> Self {
        match name {
            ScreenName::Asset => {
                let (model, remaining_apis) = asset::Model::construct(state_registry, api_registry);
                Self {
                    remaining_apis,
                    model: ScreenModel::Asset(model),
                }
            }
            ScreenName::Deposit => {
                let (model, remaining_apis) =
                    deposit::Model::construct(state_registry, api_registry);
                Self {
                    remaining_apis,
                    model: ScreenModel::Deposit(model),
                }
            }
            ScreenName::DeviceSelection => {
                let (model, remaining_apis) =
                    device_selection::Model::construct(state_registry, api_registry);
                Self {
                    remaining_apis,
                    model: ScreenModel::DeviceSelection(model),
                }
            }
            // ScreenName::Portfolio => {
            //     Self::Portfolio(portfolio::Model::construct(state_registry, api_registry))
            // }
            _ => todo!(),
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>, resources: &Resources) {
        match &self.model {
            ScreenModel::Asset(screen) => screen.render(frame, resources),
            ScreenModel::Deposit(screen) => screen.render(frame, resources),
            ScreenModel::DeviceSelection(screen) => screen.render(frame, resources),
            ScreenModel::Portfolio(screen) => screen.render(frame, resources),
        }
    }

    pub async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage> {
        match &mut self.model {
            ScreenModel::Asset(screen) => screen.tick(event).await,
            ScreenModel::Deposit(screen) => screen.tick(event).await,
            ScreenModel::DeviceSelection(screen) => screen.tick(event).await,
            ScreenModel::Portfolio(screen) => screen.tick(event).await,
        }
    }

    pub async fn deconstruct(self) -> (StateRegistry, ApiRegistry<L, C, M>) {
        match self.model {
            ScreenModel::Asset(model) => model.deconstruct(self.remaining_apis).await,
            ScreenModel::Deposit(model) => model.deconstruct(self.remaining_apis).await,
            ScreenModel::DeviceSelection(model) => model.deconstruct(self.remaining_apis).await,
            // Self::Portfolio(screen) => screen.deconstruct(),
            _ => todo!(),
        }
    }
}

trait ScreenT {
    fn render(&self, frame: &mut Frame<'_>, resources: &Resources);

    async fn tick(&mut self, event: Option<Event>) -> Option<OutgoingMessage>;
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
