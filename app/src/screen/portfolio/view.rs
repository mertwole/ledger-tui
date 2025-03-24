use bigdecimal::{BigDecimal, FromPrimitive};
use input_mapping_common::InputMappingT;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Style, Stylize},
    text::Text,
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, Row, StatefulWidget, Table, TableState,
        Widget, block::Title,
    },
};
use rust_decimal::Decimal;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use super::{Model, controller};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT,
        coin_price::CoinPriceApiT,
        common_types::{Account, Network},
    },
    screen::{
        common::{self, BackgroundWidget, network_symbol, render_centered_text},
        resources::Resources,
    },
};

pub(super) fn render<C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<C, M>,
    frame: &mut Frame<'_>,
    resources: &Resources,
) {
    frame.render_widget(
        BackgroundWidget::new(resources.background_color),
        frame.size(),
    );

    let accounts = model.state.device_accounts.as_ref();

    if let Some(accounts) = accounts {
        render_account_table(model, frame, accounts, resources);
    } else {
        render_account_table(model, frame, &[], resources);
    }

    if model.show_navigation_help {
        let mapping = controller::InputEvent::get_mapping();
        common::render_navigation_help(mapping, frame, resources);
    }
}

fn render_account_table<C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<C, M>,
    frame: &mut Frame<'_>,
    accounts: &[(Network, Vec<Account>)],
    resources: &Resources,
) {
    let area = frame.area();

    let tree_items: Vec<_> = accounts
        .iter()
        .map(|(network, accounts)| {
            let network_name = network.get_info().name;

            let leafs = accounts
                .iter()
                .map(|account| {
                    // TODO: Pretty formatting.
                    let pk = account.get_info().public_key[..8].to_string();
                    let text = Text::from(pk.clone()).fg(resources.main_color);

                    TreeItem::new_leaf(pk, text)
                })
                .collect();

            let text = Text::from(network_name.clone()).fg(resources.main_color);
            TreeItem::new(network_name, text, leafs).expect("Duplicate networks found")
        })
        .collect();

    let tree = Tree::new(&tree_items)
        .expect("Duplicate networks found")
        .highlight_style(Style::new().bold());

    let mut tree_state = TreeState::default();

    for &(network, _) in accounts {
        tree_state.open(vec![network.get_info().name]);
    }

    if let Some(network_idx) = model.selected_network {
        let (network, accounts) = &accounts[network_idx];
        let mut path = vec![network.get_info().name];

        if let Some(account_idx) = model.selected_account {
            let account = accounts[account_idx].get_info().public_key[..8].to_string();
            path.push(account);
        }

        tree_state.select(path);
    }

    frame.render_stateful_widget(tree, area, &mut tree_state);
}

fn mul_bigdecimal_decimal(lhs: BigDecimal, rhs: Decimal) -> BigDecimal {
    lhs * BigDecimal::from_f64(rhs.try_into().expect("Failed to convert Decimal to f64"))
        .expect("Fauiled to convert f64 to BigDecimal")
}
