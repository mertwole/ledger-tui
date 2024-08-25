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
use rust_decimal::Decimal;
use strum::IntoEnumIterator;

use crate::{
    api::{
        blockchain_monitoring::{
            BlockchainMonitoringApiT, TransactionInfo, TransactionType, TransactionUid,
        },
        coin_price::CoinPriceApiT,
        common_types::{Account, Network},
        ledger::LedgerApiT,
    },
    screen::{
        common::{format_address, network_symbol, render_centered_text, BackgroundWidget},
        resources::Resources,
    },
};

use super::{Model, TimePeriod};

const ADDRESSES_MAX_LEN: usize = 12;
const TX_UID_MAX_LEN: usize = 16;

pub(super) fn render<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<L, C, M>,
    frame: &mut Frame<'_>,
    resources: &Resources,
) {
    let area = frame.size();

    frame.render_widget(BackgroundWidget::new(resources.background_color), area);

    let [price_chart_area, txs_list_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1); 2])
        .areas(area);

    let price_chart_block = Block::new()
        .title("Price")
        .borders(Borders::all())
        .fg(resources.main_color);

    let inner_price_chart_area = price_chart_block.inner(price_chart_area);
    frame.render_widget(price_chart_block, price_chart_area);

    if let Some(prices) = model.coin_price_history.as_ref() {
        render_price_chart(
            &prices[..],
            model.selected_time_period,
            frame,
            inner_price_chart_area,
            resources,
        );
    } else {
        render_price_chart_placeholder(
            model.selected_time_period,
            frame,
            inner_price_chart_area,
            resources,
        );
    }

    let txs_list_block = Block::new()
        .title("Transactions")
        .borders(Borders::all())
        .padding(Padding::proportional(1))
        .fg(resources.main_color);

    let inner_txs_list_area = txs_list_block.inner(txs_list_area);
    frame.render_widget(txs_list_block, txs_list_area);

    match model.transactions.as_ref() {
        Some(tx_list) if tx_list.is_empty() => {
            render_empty_tx_list(frame, inner_txs_list_area);
        }
        Some(tx_list) => {
            let selected_account = model
                .state
                .selected_account
                .as_ref()
                .expect("Selected accounmodelt should be present in state"); // TODO: Enforce this rule at `app` level?

            render_tx_list(
                selected_account.clone(),
                &tx_list[..],
                frame,
                inner_txs_list_area,
                resources,
            );
        }
        None => {
            render_tx_list_placeholder(frame, inner_txs_list_area);
        }
    }
}

fn render_price_chart(
    prices: &[Decimal],
    selected_time_period: TimePeriod,
    frame: &mut Frame<'_>,
    area: Rect,
    resources: &Resources,
) {
    let legend = render_chart_legend(selected_time_period, resources);

    let max_price = *prices.iter().max().expect("Empty `prices` vector provided");

    let price_data: Vec<_> = prices
        .iter()
        .enumerate()
        .map(|(idx, &price)| (idx as f64, price.try_into().unwrap()))
        .collect();

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Bar)
        .name(legend)
        .graph_type(GraphType::Line)
        .data(&price_data)];

    let x_axis = Axis::default().bounds([0.0, (price_data.len() - 1) as f64]);

    let y_axis = Axis::default().bounds([0.0, max_price.try_into().unwrap()]);

    let chart = Chart::new(datasets)
        .x_axis(x_axis)
        .y_axis(y_axis)
        .legend_position(Some(ratatui::widgets::LegendPosition::BottomRight))
        // Always show a legend(see `hidden_legend_constraints` docs).
        .hidden_legend_constraints((Constraint::Min(0), Constraint::Min(0)))
        .bg(resources.background_color);

    frame.render_widget(chart, area);
}

fn render_price_chart_placeholder(
    selected_time_period: TimePeriod,
    frame: &mut Frame<'_>,
    area: Rect,
    resources: &Resources,
) {
    let legend = render_chart_legend(selected_time_period, resources);

    let chart = Chart::new(vec![Dataset::default().name(legend)])
        .legend_position(Some(ratatui::widgets::LegendPosition::BottomRight))
        // Always show a legend(see `hidden_legend_constraints` docs).
        .hidden_legend_constraints((Constraint::Min(0), Constraint::Min(0)));

    frame.render_widget(chart, area);

    let text = Text::raw("Price is loading...");
    render_centered_text(frame, area, text);
}

fn render_chart_legend(selected_time_period: TimePeriod, resources: &Resources) -> Line<'static> {
    let legend = TimePeriod::iter().map(|period| {
        let label = match period {
            TimePeriod::Day => " d[ay]",
            TimePeriod::Week => " w[eek]",
            TimePeriod::Month => " m[onth]",
            TimePeriod::Year => " y[ear]",
            TimePeriod::All => " a[ll] ",
        };

        if period == selected_time_period {
            Span::raw(label).fg(resources.accent_color)
        } else {
            Span::raw(label).fg(resources.main_color)
        }
    });

    Line::from_iter(legend)
}

fn render_tx_list(
    selected_account: (Network, Account),
    tx_list: &[(TransactionUid, TransactionInfo)],
    frame: &mut Frame<'_>,
    area: Rect,
    resources: &Resources,
) {
    let (selected_account_network, selected_account) = selected_account;

    let selected_account_address = selected_account.get_info().pk;

    let network_icon = network_symbol(selected_account_network);

    let rows = tx_list
        .iter()
        .map(|(uid, tx)| {
            let uid = format_address(&uid.uid, TX_UID_MAX_LEN);
            let uid = Text::raw(uid).alignment(Alignment::Center);

            let time = format!("{}", tx.timestamp.format("%Y-%m-%d %H:%M UTC%:z"));
            let time = Text::raw(time)
                .alignment(Alignment::Center)
                .fg(resources.secondary_color);

            let description = match &tx.ty {
                TransactionType::Deposit { from, amount } => {
                    let from = format_address(&from.get_info().pk, ADDRESSES_MAX_LEN);
                    let to = format_address(&selected_account_address, ADDRESSES_MAX_LEN);

                    vec![
                        Span::raw(from),
                        Span::raw(" -> "),
                        Span::raw(to).fg(resources.accent_color),
                        Span::raw(" for ").fg(resources.secondary_color),
                        Span::raw(format!("{}{}", amount, network_icon)),
                    ]
                }
                TransactionType::Withdraw { to, amount } => {
                    let from = format_address(&selected_account_address, ADDRESSES_MAX_LEN);
                    let to = format_address(&to.get_info().pk, ADDRESSES_MAX_LEN);

                    vec![
                        Span::raw(from).fg(resources.accent_color),
                        Span::raw(" -> "),
                        Span::raw(to),
                        Span::raw(" for ").fg(resources.secondary_color),
                        Span::raw(format!("{}{}", amount, network_icon)),
                    ]
                }
            };
            let line = Line::from_iter(description);
            let description = Text::from(line).alignment(Alignment::Left);

            Row::new(vec![description, time, uid])
        })
        .intersperse(Row::new(vec!["", "", ""]));

    let table = Table::new(rows, [Constraint::Ratio(1, 3); 3])
        .highlight_style(Style::new().reversed())
        .highlight_spacing(HighlightSpacing::WhenSelected)
        .highlight_symbol(">>");

    frame.render_widget(table, area)
}

fn render_empty_tx_list(frame: &mut Frame<'_>, area: Rect) {
    let text = Text::raw("No transactions here yet");
    render_centered_text(frame, area, text)
}

fn render_tx_list_placeholder(frame: &mut Frame<'_>, area: Rect) {
    let text = Text::raw("Fetching transactions...");
    render_centered_text(frame, area, text)
}
