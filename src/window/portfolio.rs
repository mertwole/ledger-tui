use std::{cell::RefCell, time::Duration};

use futures::executor::block_on;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, HighlightSpacing, Padding, Row, Table, TableState},
    Frame,
};

use crate::{
    api::{
        coin_price::{Coin, CoinPriceApiT},
        ledger::{Account, LedgerApiT, Network},
    },
    app::StateRegistry,
    window::WindowName,
};

use super::{EventExt, OutgoingMessage, Window};

pub struct Portfolio<L: LedgerApiT, C: CoinPriceApiT> {
    ledger_api: L,
    coin_price_api: C,

    state: Option<StateRegistry>,

    table_state: RefCell<TableState>,
}

impl<L: LedgerApiT, C: CoinPriceApiT> Window for Portfolio<L, C> {
    fn construct(&mut self, state: StateRegistry) {
        self.state = Some(state);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let state = self
            .state
            .as_ref()
            .expect("Construct should be called at the start of window lifetime");

        if let Some(accounts) = state.device_accounts.as_ref() {
            self.render_account_table(frame, accounts);
        } else {
            // TODO: Process case when device is connected but accounts haven't been loaded yet.
            Self::render_account_table_placeholder(frame);
        }
    }

    fn tick(&mut self) -> Option<OutgoingMessage> {
        let state = self
            .state
            .as_mut()
            .expect("Construct should be called at the start of window lifetime");

        if state.device_accounts.is_none() {
            if let Some((active_device, _)) = state.active_device.as_ref() {
                // TODO: Load at startup from config and add only on user request.
                // TODO: Filter accounts based on balance.
                state.device_accounts = Some(
                    [Network::Bitcoin, Network::Ethereum]
                        .into_iter()
                        .filter_map(|network| {
                            let accounts: Vec<_> =
                                block_on(self.ledger_api.discover_accounts(active_device, network))
                                    .collect();

                            if accounts.is_empty() {
                                None
                            } else {
                                Some((network, accounts))
                            }
                        })
                        .collect(),
                );
            }
        }

        self.process_input()
    }

    fn deconstruct(self: Box<Self>) -> StateRegistry {
        self.state
            .expect("Construct should be called at the start of window lifetime")
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT> Portfolio<L, C> {
    pub fn new(ledger_api: L, coin_price_api: C) -> Self {
        Self {
            ledger_api,
            coin_price_api,
            state: None,
            table_state: RefCell::default(),
        }
    }

    fn render_account_table(&self, frame: &mut Frame<'_>, accounts: &[(Network, Vec<Account>)]) {
        let area = frame.size();

        let table_block = Block::new()
            .border_type(BorderType::Double)
            .borders(Borders::all())
            .border_style(Color::Yellow)
            .padding(Padding::uniform(1))
            .title("Portfolio")
            .title_alignment(Alignment::Center);

        // TODO: Sort.
        let rows = accounts.iter().map(|(nw, acc)| {
            // TODO: Correctly map accounts to coins.
            let coin = match nw {
                Network::Bitcoin => Coin::BTC,
                Network::Ethereum => Coin::ETH,
            };

            let price = block_on(self.coin_price_api.get_price(coin, Coin::USDT));
            let price = price
                .map(|price| price.to_string())
                .unwrap_or_else(|| "N/A".to_string());

            Row::new(vec![
                nw.get_info().name,
                nw.get_info().symbol,
                acc.len().to_string(),
                price,
            ])
        });

        let table = Table::new(rows, [Constraint::Ratio(1, 4); 4])
            .column_spacing(1)
            .header(
                Row::new(vec!["Network", "Symbol", "Accounts", "USDT Price"])
                    .style(Style::new().bold())
                    .bottom_margin(1),
            )
            .block(table_block)
            .highlight_style(Style::new().reversed())
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>");

        frame.render_stateful_widget(table, area, &mut *self.table_state.borrow_mut());
    }

    fn render_account_table_placeholder(frame: &mut Frame<'_>) {
        let area = frame.size();

        let block = Block::new()
            .border_type(BorderType::Double)
            .borders(Borders::all())
            .border_style(Color::Yellow)
            .padding(Padding::uniform(1))
            .title("Portfolio")
            .title_alignment(Alignment::Center);

        let text_area = block.inner(area);

        let text = Text::raw("Device is not selected. Please select device(`d`)")
            .alignment(Alignment::Center);

        frame.render_widget(block, area);
        frame.render_widget(text, text_area);
    }

    fn process_input(&mut self) -> Option<OutgoingMessage> {
        if !event::poll(Duration::ZERO).unwrap() {
            return None;
        }

        let event = event::read().unwrap();

        if let Some(state) = self.state.as_mut() {
            if let Some(accounts) = state.device_accounts.as_ref() {
                if event.is_key_pressed(KeyCode::Enter) {
                    if let Some(selected_idx) = self.table_state.borrow().selected() {
                        // TODO: Don't ignore other accounts - let user choose it on portfolio window,
                        let selected = accounts[selected_idx].clone();
                        state.selected_account = Some((selected.0, selected.1[0].clone()));

                        return Some(OutgoingMessage::SwitchWindow(WindowName::Asset));
                    }
                }

                let accounts_len = accounts.len();
                self.process_table_navigation(&event, accounts_len);
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

    fn process_table_navigation(&mut self, event: &Event, accounts_len: usize) {
        if event.is_key_pressed(KeyCode::Down) {
            let selected = self
                .table_state
                .borrow_mut()
                .selected_mut()
                .as_mut()
                .map(|sel| {
                    *sel += 1;
                    if *sel >= accounts_len {
                        *sel = accounts_len - 1;
                    }
                })
                .is_some();

            if !selected {
                let new_selected = if accounts_len == 0 { None } else { Some(0) };

                self.table_state.borrow_mut().select(new_selected);
            }
        }

        if event.is_key_pressed(KeyCode::Up) {
            let selected = self
                .table_state
                .borrow_mut()
                .selected_mut()
                .as_mut()
                .map(|sel| {
                    *sel = sel.saturating_sub(1);
                })
                .is_some();

            if !selected {
                let new_selected = if accounts_len == 0 {
                    None
                } else {
                    Some(accounts_len - 1)
                };

                self.table_state.borrow_mut().select(new_selected);
            }
        }
    }
}
