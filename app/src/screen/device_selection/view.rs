use input_mapping_common::InputMappingT;
use ratatui::{
    Frame,
    layout::{Alignment, Margin, Rect},
    style::Stylize,
    text::Text,
    widgets::{Block, BorderType, Borders, List, Padding},
};

use super::{Model, controller};
use crate::{
    api::ledger::{Device, DeviceInfo, LedgerApiT},
    screen::{
        common::{self, BackgroundWidget, render_centered_text},
        resources::Resources,
    },
};

pub(super) fn render<L: LedgerApiT>(
    model: &Model<L>,
    frame: &mut Frame<'_>,
    resources: &Resources,
) {
    let area = frame.area();

    frame.render_widget(BackgroundWidget::new(resources.background_color), area);

    let list_block = Block::new()
        .border_type(BorderType::Double)
        .borders(Borders::all())
        .border_style(resources.main_color)
        .padding(Padding::uniform(1))
        .title("Select a device")
        .title_alignment(Alignment::Center)
        .fg(resources.main_color);

    let list_area = list_block.inner(area);

    frame.render_widget(list_block, area);

    if model.devices.is_empty() {
        render_device_list_placeholder(frame, list_area);
    } else {
        render_device_list(
            &model.devices,
            model.selected_device,
            frame,
            list_area,
            resources,
        );
    }

    if model.show_navigation_help {
        let mapping = controller::InputEvent::get_mapping();
        common::render_navigation_help(mapping, frame, resources);
    }
}

fn render_device_list_placeholder(frame: &mut Frame<'_>, area: Rect) {
    let text = Text::raw("No devices found. Try refreshing list [r].");
    render_centered_text(frame, area, text)
}

fn render_device_list(
    devices: &[(Device, DeviceInfo)],
    selected_device: Option<usize>,
    frame: &mut Frame<'_>,
    area: Rect,
    resources: &Resources,
) {
    let mut list_height = 0;
    let list = List::new(devices.iter().enumerate().map(|(idx, (_, info))| {
        let label = format!("Ledger {}", info.model);

        let item = Text::centered(label.into());

        let item = if Some(idx) == selected_device {
            item.bold()
                .bg(resources.accent_color)
                .fg(resources.background_color)
        } else {
            item.fg(resources.main_color)
        };

        list_height += item.height();

        item
    }));

    let margin = area.height.saturating_sub(list_height as u16) / 2;
    let area = area.inner(Margin::new(0, margin));

    frame.render_widget(list, area);
}
