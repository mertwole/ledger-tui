use input_mapping_common::InputMappingT;
use ratatui::{
    Frame,
    style::{Style, Stylize},
    text::Text,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use super::{Model, controller};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT,
        coin_price::CoinPriceApiT,
        common_types::{Account, Network},
        ledger::LedgerApiT,
        storage::StorageApiT,
    },
    screen::{
        common::{self, BackgroundWidget},
        resources::Resources,
    },
};

pub(super) fn render<
    L: LedgerApiT,
    C: CoinPriceApiT,
    M: BlockchainMonitoringApiT,
    S: StorageApiT,
>(
    model: &Model<L, C, M, S>,
    frame: &mut Frame<'_>,
    resources: &Resources,
) {
    frame.render_widget(
        BackgroundWidget::new(resources.background_color),
        frame.area(),
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

fn render_account_table<
    L: LedgerApiT,
    C: CoinPriceApiT,
    M: BlockchainMonitoringApiT,
    S: StorageApiT,
>(
    model: &Model<L, C, M, S>,
    frame: &mut Frame<'_>,
    accounts: &[(Network, Vec<Account>)],
    resources: &Resources,
) {
    let area = frame.area();

    let mut tree_items: Vec<_> = accounts
        .iter()
        .map(|(network, accounts)| {
            let network_name = network.get_info().name;

            let mut leafs: Vec<_> = accounts
                .iter()
                .map(|account| {
                    // TODO: Pretty formatting.
                    let pk = account.get_info().public_key[..8].to_string();
                    let text = Text::from(pk.clone()).fg(resources.main_color);

                    TreeItem::new_leaf(pk, text)
                })
                .collect();

            let add_account_tree_item = TreeItem::new_leaf(
                "AddAccount".to_string(),
                Text::from("+ discover accounts").fg(resources.main_color),
            );

            leafs.push(add_account_tree_item);

            let text = Text::from(network_name.clone()).fg(resources.main_color);
            TreeItem::new(network_name, text, leafs).expect("Duplicate networks found")
        })
        .collect();

    let add_network_tree_item = TreeItem::new_leaf(
        "AddNetwork".to_string(),
        Text::from("+ add networks").fg(resources.main_color),
    );

    tree_items.push(add_network_tree_item);

    let tree = Tree::new(&tree_items)
        .expect("Duplicate networks found")
        .highlight_style(Style::new().bold());

    let mut tree_state = TreeState::default();

    for &(network, _) in accounts {
        tree_state.open(vec![network.get_info().name]);
    }

    if let Some(network_idx) = model.selected_network {
        if network_idx == accounts.len() {
            tree_state.select(vec!["AddNetwork".to_string()]);
        } else {
            let (network, accounts) = &accounts[network_idx];
            let mut path = vec![network.get_info().name];

            if let Some(account_idx) = model.selected_account {
                if account_idx == accounts.len() {
                    path.push("AddAccount".to_string());
                } else {
                    let account = accounts[account_idx].get_info().public_key[..8].to_string();
                    path.push(account);
                }
            }

            tree_state.select(path);
        }
    }

    frame.render_stateful_widget(tree, area, &mut tree_state);
}
