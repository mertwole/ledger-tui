use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{
        block::Title, Block, BorderType, Borders, HighlightSpacing, Padding, Row, StatefulWidget,
        Table, TableState, Widget,
    },
    Frame,
};
use rust_decimal::Decimal;
use tui_widget_list::PreRender;

use super::Model;
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT,
        coin_price::CoinPriceApiT,
        common::{Account, Network},
        ledger::LedgerApiT,
    },
    screen::common::network_symbol,
};

pub(super) fn render<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<L, C, M>,
    frame: &mut Frame<'_>,
) {
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

fn render_account_table<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<L, C, M>,
    frame: &mut Frame<'_>,
    accounts: &[(Network, Vec<Account>)],
) {
    let area = frame.size();

    let accounts = accounts
        .iter()
        .enumerate()
        .map(|(network_idx, (network, accounts))| {
            let selected_account = match model.selected_account {
                Some((selected_network, selected_account)) if selected_network == network_idx => {
                    Some(selected_account)
                }
                _ => None,
            };

            let price = model.coin_prices.get(network).copied().unwrap_or_default();

            let accounts_and_balances: Vec<_> = accounts
                .iter()
                .map(|account| {
                    (
                        account.clone(),
                        model.balances.get(&(*network, account.clone())).copied(),
                    )
                })
                .collect();

            NetworkAccountsTable {
                network: *network,
                accounts_and_balances,
                selected_account,
                is_self_selected: false,
                price,
            }
        })
        .collect();

    let list = tui_widget_list::List::new(accounts);
    let mut state = tui_widget_list::ListState::default();

    let selected_network = model.selected_account.map(|(network, _)| network);
    state.select(selected_network);

    frame.render_stateful_widget(list, area, &mut state);
}

struct NetworkAccountsTable {
    network: Network,
    accounts_and_balances: Vec<(Account, Option<Decimal>)>,

    selected_account: Option<usize>,
    is_self_selected: bool,

    price: Option<Decimal>,
}

impl PreRender for NetworkAccountsTable {
    fn pre_render(&mut self, context: &tui_widget_list::PreRenderContext) -> u16 {
        self.is_self_selected = context.is_selected;

        self.accounts_and_balances.len() as u16 + 2
    }
}

impl Widget for NetworkAccountsTable {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let icon = network_symbol(self.network);

        let price_label = format!(
            "1{} = {}₮",
            icon,
            self.price
                .map(|price| price.to_string())
                .unwrap_or_else(|| "N/A".to_string())
        );

        let block = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::all())
            .border_style(Color::Yellow)
            .title(Title::from(self.network.get_info().name).alignment(Alignment::Left))
            .title(Title::from(price_label).alignment(Alignment::Right));

        let block = if self.is_self_selected {
            block.bold()
        } else {
            block
        };

        let rows = self.accounts_and_balances.iter().map(|(account, balance)| {
            // TODO: Pretty formatting.
            let pk = account.get_info().pk[..8].to_string();

            let price = balance
                .zip(self.price)
                .map(|(balance, price)| balance * price)
                .map(|price| format!("{}₮", price))
                .unwrap_or_else(|| "Fetching price...".to_string());

            let balance = balance
                .map(|balance| [balance.to_string(), icon.clone()].concat())
                .unwrap_or_else(|| "Fetching price...".to_string());

            let pk = Text::from(pk).alignment(Alignment::Left);
            let balance = Text::from(balance).alignment(Alignment::Center);
            let price = Text::from(price).alignment(Alignment::Right);

            Row::new(vec![pk, balance, price])
        });

        let table = Table::new(rows, [Constraint::Ratio(1, 3); 3])
            .column_spacing(1)
            .block(block)
            .highlight_style(Style::new().reversed())
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>");

        let mut table_state = TableState::default().with_selected(self.selected_account);
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
