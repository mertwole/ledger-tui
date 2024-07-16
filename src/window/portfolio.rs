use std::{cell::RefCell, collections::HashMap, time::Duration};

use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Alignment, Constraint},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, HighlightSpacing, Padding, Row, Table, TableState},
    Frame,
};

use crate::api::ledger::{Account, Device, LedgerApiT, Network};

use super::EventExt;

pub struct Portfolio<L: LedgerApiT> {
    ledger_api: L,
    ledger_device: Device,
    accounts: HashMap<Network, Vec<Account>>,

    table_state: RefCell<TableState>,
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

            table_state: RefCell::default(),
        }
    }

    pub async fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.size();

        let table_block = Block::new()
            .border_type(BorderType::Double)
            .borders(Borders::all())
            .border_style(Color::Yellow)
            .padding(Padding::uniform(1))
            .title("Portfolio")
            .title_alignment(Alignment::Center);

        // TODO: Sort.
        let rows = self.accounts.iter().map(|(nw, acc)| {
            Row::new(vec![
                nw.get_info().name,
                nw.get_info().symbol,
                acc.len().to_string(),
            ])
        });

        let table = Table::new(rows, [Constraint::Ratio(1, 3); 3])
            .column_spacing(1)
            .header(
                Row::new(vec!["Network", "Symbol", "Accounts"])
                    .style(Style::new().bold())
                    .bottom_margin(1),
            )
            .block(table_block)
            .highlight_style(Style::new().reversed())
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>");

        let mut table_state = self.table_state.borrow_mut();
        frame.render_stateful_widget(table, area, &mut *table_state);
    }

    pub async fn tick(&mut self) -> Option<OutgoingMessage> {
        // TODO: Load at startup from config and add only on user request.
        // TODO: Filter accounts based on balance.
        for network in [Network::Bitcoin, Network::Ethereum] {
            let accs = self
                .ledger_api
                .discover_accounts(&self.ledger_device, network)
                .await
                .collect();

            self.accounts.entry(network).or_insert(accs);
        }

        self.process_input().await
    }

    async fn process_input(&mut self) -> Option<OutgoingMessage> {
        if !event::poll(Duration::ZERO).unwrap() {
            return None;
        }

        let event = event::read().unwrap();

        log::info!("Accounts len: {}", self.accounts.len());

        if event.is_key_pressed(KeyCode::Down) {
            let selected = self
                .table_state
                .borrow_mut()
                .selected_mut()
                .as_mut()
                .map(|sel| {
                    *sel += 1;
                    if *sel >= self.accounts.len() {
                        *sel = self.accounts.len() - 1;
                    }
                })
                .is_some();

            if !selected {
                let new_selected = if self.accounts.is_empty() {
                    None
                } else {
                    Some(0)
                };

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
                let new_selected = if self.accounts.is_empty() {
                    None
                } else {
                    Some(self.accounts.len() - 1)
                };

                self.table_state.borrow_mut().select(new_selected);
            }
        }

        if event.is_key_pressed(KeyCode::Char('q')) {
            return Some(OutgoingMessage::Quit);
        }

        None
    }
}
