use ratatui::Frame;

use crate::api::ledger::{Device, LedgerApiT};

pub struct Portfolio<L: LedgerApiT> {
    ledger_api: L,
    ledger_device: Device,
}

pub enum OutgoingMessage {
    Quit,
}

impl<L: LedgerApiT> Portfolio<L> {
    pub async fn new(ledger_api: L, ledger_device: Device) -> Self {
        Self {
            ledger_api,
            ledger_device,
        }
    }

    pub async fn render(&self, frame: &mut Frame<'_>) {
        //
    }

    pub async fn tick(&mut self) -> Option<OutgoingMessage> {
        None
    }
}
