use std::time::Duration;

use ratatui::{
    crossterm::event::{self, KeyCode},
    text::Text,
    Frame,
};

use crate::{
    api::{coin_price::CoinPriceApiT, ledger::LedgerApiT},
    app::StateRegistry,
};

use super::{EventExt, OutgoingMessage, Window};

pub struct Asset<L: LedgerApiT, C: CoinPriceApiT> {
    ledger_api: L,
    coin_price_api: C,

    state: Option<StateRegistry>,
}

impl<L: LedgerApiT, C: CoinPriceApiT> Window for Asset<L, C> {
    fn construct(&mut self, state: StateRegistry) {
        self.state = Some(state);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.size();

        let text = Text::raw("Test");

        frame.render_widget(text, area);
    }

    fn tick(&mut self) -> Option<OutgoingMessage> {
        self.process_input()
    }

    fn deconstruct(self: Box<Self>) -> StateRegistry {
        self.state
            .expect("Construct should be called at the start of window lifetime")
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT> Asset<L, C> {
    pub fn new(ledger_api: L, coin_price_api: C) -> Self {
        Self {
            ledger_api,
            coin_price_api,
            state: None,
        }
    }

    fn process_input(&mut self) -> Option<OutgoingMessage> {
        if !event::poll(Duration::ZERO).unwrap() {
            return None;
        }

        let event = event::read().unwrap();

        if event.is_key_pressed(KeyCode::Char('q')) {
            return Some(OutgoingMessage::Exit);
        }

        None
    }
}
