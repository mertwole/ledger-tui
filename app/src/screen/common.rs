use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    text::Text,
    Frame,
};

use crate::api::common_types::Network;

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

#[cfg(test)]
mod tests {
    use std::iter;

    use itertools::Itertools;

    use super::format_address;

    #[test]
    fn test_format_address() {
        let address_lengths = [0, 2, 8, 10, 100].into_iter();
        let max_lengths = [0, 3, 5, 6, 8, 10, 100].into_iter();

        for (addr_len, max_len) in address_lengths.cartesian_product(max_lengths) {
            let address: String = iter::repeat('0').take(addr_len).collect();
            assert!(format_address(&address, max_len).len() <= max_len);
        }
    }
}
