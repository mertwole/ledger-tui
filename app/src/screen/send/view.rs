use input_mapping_common::InputMappingT;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::Stylize,
    text::Text,
};

use crate::{
    api::ledger::LedgerApiT,
    screen::{
        common::{self, BackgroundWidget},
        resources::Resources,
    },
};

use super::Model;

pub(super) fn render<L: LedgerApiT>(
    model: &Model<L>,
    frame: &mut Frame<'_>,
    resources: &Resources,
) {
    let area = frame.area();

    frame.render_widget(BackgroundWidget::new(resources.background_color), area);

    let sender = model
        .state
        .selected_account
        .as_ref()
        .expect("Selected account should be present in state") // TODO: Enforce this rule at `app` level?
        .1
        .get_info()
        .public_key;
    let sender = Text::from(sender).fg(resources.main_color);
    let sender_label = Text::from("sender:").fg(resources.main_color);

    let receiver = if let Some(receiver_address) = &model.receiver_address {
        Text::from(&**receiver_address).fg(resources.main_color)
    } else {
        Text::from("paste receiver address [p]").fg(resources.accent_color)
    };
    let receiver_label = Text::from("receiver:").fg(resources.main_color);

    let amount = if let Some(amount) = &model.send_amount {
        Text::from(amount.to_string()).fg(resources.main_color)
    } else {
        Text::from("start typing amount").fg(resources.accent_color)
    };
    let amount_label = Text::from("amount:").fg(resources.main_color);

    let [
        sender_label_area,
        sender_area,
        _,
        receiver_label_area,
        receiver_area,
        _,
        amount_label_area,
        amount_area,
    ] = Layout::vertical([
        Constraint::Length(sender_label.height() as u16),
        Constraint::Length(sender.height() as u16),
        Constraint::Length(1),
        Constraint::Length(receiver_label.height() as u16),
        Constraint::Length(receiver.height() as u16),
        Constraint::Length(1),
        Constraint::Length(amount_label.height() as u16),
        Constraint::Length(amount.height() as u16),
    ])
    .flex(Flex::Center)
    .areas(area);

    frame.render_widget(sender_label.centered(), sender_label_area);
    frame.render_widget(sender.centered(), sender_area);
    frame.render_widget(receiver_label.centered(), receiver_label_area);
    frame.render_widget(receiver.centered(), receiver_area);
    frame.render_widget(amount_label.centered(), amount_label_area);
    frame.render_widget(amount.centered(), amount_area);

    if model.show_navigation_help {
        let mapping = super::controller::InputEvent::get_mapping();
        common::render_navigation_help(mapping, frame, resources);
    }
}
