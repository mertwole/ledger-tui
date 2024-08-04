use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, HighlightSpacing, Padding, Row, Table,
    },
    Frame,
};

use crate::{
    api::{
        blockchain_monitoring::{BlockchainMonitoringApiT, TransactionType},
        coin_price::CoinPriceApiT,
        ledger::LedgerApiT,
    },
    screen::common::network_symbol,
};

use super::Model;

pub(super) fn render<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<L, C, M>,
    frame: &mut Frame<'_>,
) {
    let area = frame.size();

    let [price_chart_area, txs_list_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1); 2])
        .areas(area);

    let price_chart_block = Block::new().title("Price").borders(Borders::all());
    let inner_price_chart_area = price_chart_block.inner(price_chart_area);
    frame.render_widget(price_chart_block, price_chart_area);
    render_price_chart(model, frame, inner_price_chart_area);

    let txs_list_block = Block::new()
        .title("Transactions")
        .borders(Borders::all())
        .padding(Padding::proportional(1));
    let inner_txs_list_area = txs_list_block.inner(txs_list_area);
    frame.render_widget(txs_list_block, txs_list_area);
    render_tx_list(model, frame, inner_txs_list_area);
}

fn render_price_chart<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<L, C, M>,
    frame: &mut Frame<'_>,
    area: Rect,
) {
    let Some(prices) = model.coin_price_history.as_ref() else {
        // TODO: Draw some placeholder.
        return;
    };

    let mut price_bounds = [f64::MAX, f64::MIN];
    for (_, price) in prices {
        let price: f64 = (*price).try_into().unwrap();
        price_bounds[0] = price_bounds[0].min(price);
        price_bounds[1] = price_bounds[1].max(price);
    }

    let price_data: Vec<_> = prices
        .iter()
        .enumerate()
        .map(|(idx, price)| (idx as f64, price.1.try_into().unwrap()))
        .collect();

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Bar)
        .graph_type(GraphType::Line)
        .style(Style::default().magenta())
        .data(&price_data)];

    let x_axis = Axis::default()
        .style(Style::default().white())
        .bounds([0.0, price_data.len() as f64]);

    let y_axis = Axis::default()
        .style(Style::default().white())
        .bounds(price_bounds);

    let chart = Chart::new(datasets).x_axis(x_axis).y_axis(y_axis);

    frame.render_widget(chart, area);
}

fn render_tx_list<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<L, C, M>,
    frame: &mut Frame<'_>,
    area: Rect,
) {
    let Some(tx_list) = model.transactions.as_ref() else {
        // TODO: Draw placeholder(fetching txs...)
        return;
    };

    if tx_list.is_empty() {
        // TODO: Draw placeholder(no txs yet...)
        return;
    }

    let state = model
        .state
        .as_ref()
        .expect("Construct should be called at the start of window lifetime");

    let (selected_account_network, selected_account) = state
        .selected_account
        .as_ref()
        .expect("Selected account should be present in state"); // TODO: Enforce this rule at `app` level?

    let selected_account_address = selected_account.get_info().pk;

    let network_icon = network_symbol(*selected_account_network);

    let rows = tx_list.iter().map(|(uid, tx)| {
        // TODO: Pretty-format.
        let uid = [&uid.uid[..8], "..."].concat();
        let uid = Text::raw(uid).alignment(Alignment::Center);

        let description = match &tx.ty {
            TransactionType::Deposit { from, amount } => {
                // TODO: Pretty-format.
                let from = [&from.get_info().pk[..8], "..."].concat();
                let to = [&selected_account_address[..8], "..."].concat();

                vec![
                    Span::raw(from),
                    Span::raw(" -> "),
                    Span::raw(to).green(),
                    Span::raw(format!(" for {}{}", amount.to_string(), network_icon)),
                ]
            }
            TransactionType::Withdraw { to, amount } => {
                // TODO: Pretty-format.
                let from = [&selected_account_address[..8], "..."].concat();
                let to = [&to.get_info().pk[..8], "..."].concat();

                vec![
                    Span::raw(from).green(),
                    Span::raw(" -> "),
                    Span::raw(to),
                    Span::raw(format!(" for {}{}", amount.to_string(), network_icon)),
                ]
            }
        };
        let line = Line::from_iter(description.into_iter());
        let description = Text::from(line).alignment(Alignment::Left);

        Row::new(vec![description, uid])
    });

    let table = Table::new(rows, [Constraint::Ratio(1, 2); 2])
        .highlight_style(Style::new().reversed())
        .highlight_spacing(HighlightSpacing::WhenSelected)
        .highlight_symbol(">>");

    frame.render_widget(table, area)
}
