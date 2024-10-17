use input_mapping_common::InputMappingT;
use ratatui::{
    layout::{Alignment, Margin},
    style::Stylize,
    text::Text,
    widgets::{Block, BorderType, Borders, List, Padding},
    Frame,
};

use super::{controller, Model};
use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        ledger::LedgerApiT,
    },
    screen::{
        common::{self, BackgroundWidget},
        resources::Resources,
    },
};

pub(super) fn render<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<L, C, M>,
    frame: &mut Frame<'_>,
    resources: &Resources,
) {
    let area = frame.size();

    frame.render_widget(BackgroundWidget::new(resources.background_color), area);

    let list_block = Block::new()
        .border_type(BorderType::Double)
        .borders(Borders::all())
        .border_style(resources.main_color)
        .padding(Padding::uniform(1))
        .title("Select a device")
        .title_alignment(Alignment::Center);

    let mut list_height = 0;
    let list = List::new(
        model
            .devices
            .lock()
            .expect("Failed to acquire lock on mutex")
            .iter()
            .enumerate()
            .map(|(idx, (_, info))| {
                let label = format!(
                    "{} MCU v{} SE v{}",
                    info.model, info.mcu_version, info.se_version
                );

                let item = Text::centered(label.into());

                let item = if Some(idx) == model.selected_device {
                    item.bold()
                        .bg(resources.accent_color)
                        .fg(resources.background_color)
                } else {
                    item.fg(resources.main_color)
                };

                list_height += item.height();

                item
            }),
    );

    let list_area = list_block.inner(area);
    let margin = list_area.height.saturating_sub(list_height as u16) / 2;
    let list_area = list_area.inner(Margin::new(0, margin));

    frame.render_widget(list_block, area);
    frame.render_widget(list, list_area);

    if model.show_navigation_help {
        let mapping = controller::InputEvent::get_mapping();
        common::render_navigation_help(mapping, frame, resources);
    }
}
