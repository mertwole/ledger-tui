use std::{collections::HashMap, time::Duration};

use ratatui::{
    crossterm::event::{self, KeyCode},
    Frame,
};

use crate::api::ledger::{Account, Device, LedgerApiT, Network};

use super::EventExt;

pub struct Portfolio<L: LedgerApiT> {
    ledger_api: L,
    ledger_device: Device,
    accounts: HashMap<Network, Vec<Account>>,
}

pub enum OutgoingMessage {
    Quit,
}

impl<L: LedgerApiT> Portfolio<L> {
    pub async fn new(ledger_api: L, ledger_device: Device) -> Self {
        Self {
            ledger_api,
            ledger_device,
            accounts: HashMap::new(),
        }
    }

    pub async fn render(&self, frame: &mut Frame<'_>) {
        //
    }

    pub async fn tick(&mut self) -> Option<OutgoingMessage> {
        // TODO: Load at startup from config and add only on user request.
        // TODO: Filter accounts based on balance.
        let btc_accs = self
            .ledger_api
            .discover_accounts(&self.ledger_device, Network::Bitcoin)
            .await
            .collect();

        if !self.accounts.contains_key(&Network::Bitcoin) {
            self.accounts.insert(Network::Bitcoin, btc_accs);
        }

        self.process_input().await
    }

    async fn process_input(&mut self) -> Option<OutgoingMessage> {
        if !event::poll(Duration::ZERO).unwrap() {
            return None;
        }

        let event = event::read().unwrap();

        if event.is_key_pressed(KeyCode::Char('q')) {
            return Some(OutgoingMessage::Quit);
        }

        None
    }
}
