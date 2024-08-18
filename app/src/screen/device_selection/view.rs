use ratatui::{
    layout::{Alignment, Margin},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, List, Padding},
    Frame,
};

use super::Model;
use crate::api::{
    blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT, ledger::LedgerApiT,
};

pub(super) fn render<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<L, C, M>,
    frame: &mut Frame<'_>,
) {
    let area = frame.size();

    let list_block = Block::new()
        .border_type(BorderType::Double)
        .borders(Borders::all())
        .border_style(Color::Green)
        .padding(Padding::uniform(1))
        .title("Select a device")
        .title_alignment(Alignment::Center);

    let mut list_height = 0;
    let list = List::new(model.devices.iter().enumerate().map(|(idx, (_, info))| {
        let label = format!(
            "{} MCU v{} SE v{}",
            info.model, info.mcu_version, info.se_version
        );

        let mut item = Text::centered(label.into());

        if Some(idx) == model.selected_device {
            item = item.bold().bg(Color::DarkGray);
        }

        list_height += item.height();

        item
    }));

    let list_area = list_block.inner(area);
    let margin = list_area.height.saturating_sub(list_height as u16) / 2;
    let list_area = list_area.inner(Margin::new(0, margin));

    frame.render_widget(list_block, area);
    frame.render_widget(list, list_area);
}
