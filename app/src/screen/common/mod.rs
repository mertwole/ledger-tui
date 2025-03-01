use input_mapping_common::InputMapping;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Margin, Rect},
    style::Stylize,
    text::Text,
    widgets::{Block, BorderType, Borders, Padding},
};

use crate::api::common_types::Network;

mod background_widget;
pub use background_widget::*;

mod navigation_help_widget;
pub use navigation_help_widget::*;

use super::resources::Resources;

pub fn network_symbol(network: Network) -> String {
    match network {
        Network::Bitcoin => "₿",
        Network::Ethereum => "⟠",
    }
    .to_string()
}

pub fn render_centered_text(frame: &mut Frame, area: Rect, text: Text) {
    let [area] = Layout::horizontal([Constraint::Length(text.width() as u16)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(text.height() as u16)])
        .flex(Flex::Center)
        .areas(area);

    frame.render_widget(text, area);
}

pub fn format_address(address: &str, max_symbols: usize) -> String {
    if max_symbols <= 3 {
        return "".to_string();
    }

    if max_symbols <= 8 {
        return "...".to_string();
    }

    let part_size = (max_symbols - 3) / 2;
    let part_size = part_size.min(8);

    if address.len() <= part_size * 2 {
        return address.to_string();
    }

    format!(
        "{}...{}",
        &address[..part_size],
        &address[(address.len() - part_size)..]
    )
}

pub fn render_navigation_help(
    input_mapping: InputMapping,
    frame: &mut Frame<'_>,
    resources: &Resources,
) {
    let area = frame.size();

    let bindings = input_mapping
        .mapping
        .into_iter()
        .map(|map| (map.key, map.description))
        .collect();

    let widget = NavigationHelpWidget::new(bindings);

    let block_area = area.inner(Margin::new(8, 4));

    let width = widget.min_width().max(block_area.width as usize / 2);
    let height = widget.height();

    let block = Block::new()
        .border_type(BorderType::Double)
        .borders(Borders::all())
        .border_style(resources.main_color)
        .padding(Padding::proportional(1))
        .title("Help")
        .title_alignment(Alignment::Center)
        .reset()
        .bg(resources.background_color)
        .fg(resources.main_color);

    let block_inner = block.inner(block_area);

    let [widget_area] = Layout::horizontal([Constraint::Length(width as u16)])
        .flex(Flex::Center)
        .areas(block_inner);
    let [widget_area] = Layout::vertical([Constraint::Length(height as u16)])
        .flex(Flex::Center)
        .areas(widget_area);

    frame.render_widget(
        BackgroundWidget::new(resources.background_color),
        block_area,
    );
    frame.render_widget(block, block_area);

    frame.render_widget(widget, widget_area);
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::format_address;

    #[test]
    fn test_format_address() {
        let address_lengths = [0, 2, 8, 10, 100].into_iter();
        let max_lengths = [0, 3, 5, 6, 8, 10, 100].into_iter();

        for (addr_len, max_len) in address_lengths.cartesian_product(max_lengths) {
            let address = "0".repeat(addr_len);
            assert!(format_address(&address, max_len).len() <= max_len);
        }
    }
}
