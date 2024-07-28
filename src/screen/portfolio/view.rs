use futures::executor::block_on;
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, Padding, Row, StatefulWidget, Table,
        TableState, Widget,
    },
    Frame,
};
use tui_widget_list::PreRender;

use super::Model;
use crate::api::{
    coin_price::{Coin, CoinPriceApiT},
    ledger::{Account, LedgerApiT, Network},
};

pub(super) fn render<L: LedgerApiT, C: CoinPriceApiT>(model: &Model<L, C>, frame: &mut Frame<'_>) {
    let model_state = model
        .state
        .as_ref()
        .expect("Construct should be called at the start of window lifetime");

    if let Some(accounts) = model_state.device_accounts.as_ref() {
        render_account_table(model, frame, accounts);
    } else {
        // TODO: Process case when device is connected but accounts haven't been loaded yet.
        render_account_table_placeholder(frame);
    }
}

fn render_account_table<L: LedgerApiT, C: CoinPriceApiT>(
    model: &Model<L, C>,
    frame: &mut Frame<'_>,
    accounts: &[(Network, Vec<Account>)],
) {
    let area = frame.size();

    let accounts = accounts
        .iter()
        .map(|(network, accounts)| NetworkAccountsTable {
            network: *network,
            accounts,
            model,
        })
        .collect();

    let list = tui_widget_list::List::new(accounts);
    let mut state = tui_widget_list::ListState::default();

    frame.render_stateful_widget(list, area, &mut state);
}

struct NetworkAccountsTable<'a, L: LedgerApiT, C: CoinPriceApiT> {
    network: Network,
    accounts: &'a [Account],
    model: &'a Model<L, C>,
}

impl<L: LedgerApiT, C: CoinPriceApiT> PreRender for NetworkAccountsTable<'_, L, C> {
    fn pre_render(&mut self, _context: &tui_widget_list::PreRenderContext) -> u16 {
        self.accounts.len() as u16 + 4
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT> Widget for NetworkAccountsTable<'_, L, C> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::new()
            .border_type(BorderType::Double)
            .borders(Borders::all())
            .border_style(Color::Yellow)
            .padding(Padding::uniform(1))
            .title(self.network.get_info().name)
            .title_alignment(Alignment::Center);

        let rows = self.accounts.iter().map(|acc| {
            // TODO: Correctly map accounts to coins.
            let (coin, icon) = match self.network {
                Network::Bitcoin => (Coin::BTC, "₿"),
                Network::Ethereum => (Coin::ETH, "⟠"),
            };
            // TODO: Move API call to model.
            let price = block_on(self.model.coin_price_api.get_price(coin, Coin::USDT));
            let price = price
                .map(|price| price.to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let pk = acc.get_info().pk[..8].to_string();

            let balance = ["0.0000", icon].concat();

            Row::new(vec![pk, balance, price])
        });

        let table = Table::new(rows, [Constraint::Ratio(1, 3); 3])
            .column_spacing(1)
            .block(block)
            .highlight_style(Style::new().reversed())
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>");

        let mut table_state = TableState::default().with_selected(self.model.selected_account);
        StatefulWidget::render(table, area, buf, &mut table_state);
    }
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

    let text =
        Text::raw("Device is not selected. Please select device(`d`)").alignment(Alignment::Center);

    frame.render_widget(block, area);
    frame.render_widget(text, text_area);
}
