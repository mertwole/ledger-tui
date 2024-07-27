use futures::executor::block_on;
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, HighlightSpacing, Padding, Row, Table, TableState},
    Frame,
};

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

        let price = block_on(model.coin_price_api.get_price(coin, Coin::USDT));
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

    let mut table_state = TableState::default().with_selected(model.selected_account);
    frame.render_stateful_widget(table, area, &mut table_state);
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
