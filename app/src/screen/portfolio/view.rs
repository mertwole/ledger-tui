use bigdecimal::{BigDecimal, FromPrimitive};
use input_mapping_common::InputMappingT;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Style, Stylize},
    text::Text,
    widgets::{
        block::Title, Block, BorderType, Borders, HighlightSpacing, Padding, Row, StatefulWidget,
        Table, TableState, Widget,
    },
    Frame,
};
use rust_decimal::Decimal;
use tui_widget_list::PreRender;

use super::{controller, Model};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT,
        coin_price::CoinPriceApiT,
        common_types::{Account, Network},
        ledger::LedgerApiT,
    },
    screen::{
        common::{self, network_symbol, BackgroundWidget},
        resources::Resources,
    },
};

pub(super) fn render<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<L, C, M>,
    frame: &mut Frame<'_>,
    resources: &Resources,
) {
    frame.render_widget(
        BackgroundWidget::new(resources.background_color),
        frame.size(),
    );

    if let Some(accounts) = model.state.device_accounts.as_ref() {
        render_account_table(model, frame, accounts, resources);
    } else {
        // TODO: Process case when device is connected but accounts haven't been loaded yet.
        render_account_table_placeholder(frame, resources);
    }

    if model.show_navigation_help {
        let mapping = controller::InputEvent::get_mapping();
        common::render_navigation_help(mapping, frame, resources);
    }
}

fn render_account_table<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<L, C, M>,
    frame: &mut Frame<'_>,
    accounts: &[(Network, Vec<Account>)],
    resources: &Resources,
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

            let price = model
                .coin_prices
                .lock()
                .unwrap()
                .get(network)
                .copied()
                .unwrap_or_default();

            let accounts_and_balances: Vec<_> = accounts
                .iter()
                .map(|account| {
                    (
                        account.clone(),
                        model
                            .balances
                            .lock()
                            .expect("Failed to acquire lock on mutex")
                            .get(&(*network, account.clone()))
                            .cloned(),
                    )
                })
                .collect();

            NetworkAccountsTable {
                network: *network,
                accounts_and_balances,
                selected_account,
                is_self_selected: false,
                price,
                resources,
            }
        })
        .collect();

    let list = tui_widget_list::List::new(accounts);
    let mut state = tui_widget_list::ListState::default();

    let selected_network = model.selected_account.map(|(network, _)| network);
    state.select(selected_network);

    frame.render_stateful_widget(list, area, &mut state);
}

struct NetworkAccountsTable<'a> {
    network: Network,
    accounts_and_balances: Vec<(Account, Option<BigDecimal>)>,

    selected_account: Option<usize>,
    is_self_selected: bool,

    price: Option<Decimal>,

    resources: &'a Resources,
}

impl<'a> PreRender for NetworkAccountsTable<'a> {
    fn pre_render(&mut self, context: &tui_widget_list::PreRenderContext) -> u16 {
        self.is_self_selected = context.is_selected;

        self.accounts_and_balances.len() as u16 + 2
    }
}

impl<'a> Widget for NetworkAccountsTable<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
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
            .border_style(self.resources.main_color)
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
                .clone()
                .zip(self.price)
                .map(|(balance, price)| mul_bigdecimal_decimal(balance, price))
                .map(|price| format!("{}₮", price.round(10)))
                .unwrap_or_else(|| "Fetching price...".to_string());

            let balance = balance
                .clone()
                .map(|balance| [balance.to_string(), icon.clone()].concat())
                .unwrap_or_else(|| "Fetching price...".to_string());

            let pk = Text::from(pk).alignment(Alignment::Left);
            let balance = Text::from(balance).alignment(Alignment::Center);
            let price = Text::from(price).alignment(Alignment::Right);

            Row::new(vec![pk, balance, price]).fg(self.resources.main_color)
        });

        let table = Table::new(rows, [Constraint::Ratio(1, 3); 3])
            .column_spacing(1)
            .block(block)
            .highlight_style(
                Style::new()
                    .bg(self.resources.accent_color)
                    .fg(self.resources.background_color),
            )
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>");

        let mut table_state = TableState::default().with_selected(self.selected_account);
        StatefulWidget::render(table, area, buf, &mut table_state);
    }
}

fn render_account_table_placeholder(frame: &mut Frame<'_>, resources: &Resources) {
    let area = frame.size();

    let block = Block::new()
        .border_type(BorderType::Double)
        .borders(Borders::all())
        .border_style(resources.main_color)
        .padding(Padding::uniform(1))
        .title("Portfolio")
        .title_alignment(Alignment::Center);

    let text_area = block.inner(area);

    let text = Text::raw("Device is not selected. Please select device(`d`)")
        .alignment(Alignment::Center)
        .fg(resources.main_color);

    frame.render_widget(block, area);
    frame.render_widget(text, text_area);
}

fn mul_bigdecimal_decimal(lhs: BigDecimal, rhs: Decimal) -> BigDecimal {
    lhs * BigDecimal::from_f64(rhs.try_into().expect("Failed to convert Decimal to f64"))
        .expect("Fauiled to convert f64 to BigDecimal")
}
