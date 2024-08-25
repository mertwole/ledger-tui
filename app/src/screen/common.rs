use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::Widget,
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

pub struct BackgroundWidget {
    color: Color,
}

impl BackgroundWidget {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Widget for BackgroundWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let line = Line::raw(" ".repeat(area.width as usize)).bg(self.color);
        for y in area.y..area.y + area.height {
            buf.set_line(area.x, y, &line, area.width);
        }
    }
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
