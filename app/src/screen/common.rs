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
