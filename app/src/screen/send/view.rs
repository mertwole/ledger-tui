use std::time::Duration;

use input_mapping_common::InputMappingT;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    prelude::Buffer,
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, Padding, Widget},
};

use crate::{
    api::ledger::LedgerApiT,
    screen::{
        common::{self, BackgroundWidget},
        resources::Resources,
    },
};

use super::Model;

const DISPLAY_COPIED_TEXT_FOR: Duration = Duration::from_secs(2);

pub(super) fn render<L: LedgerApiT>(
    model: &Model<L>,
    frame: &mut Frame<'_>,
    resources: &Resources,
) {
    let area = frame.area();

    frame.render_widget(BackgroundWidget::new(resources.background_color), area);

    let pubkey = model
        .state
        .selected_account
        .as_ref()
        .expect("Selected account should be present in state") // TODO: Enforce this rule at `app` level?
        .1
        .get_info()
        .public_key;

    let address_text = Text::raw(&pubkey)
        .alignment(Alignment::Center)
        .fg(resources.main_color);

    let display_copied_text = if let Some(last_copy) = model.last_address_copy {
        last_copy.elapsed() <= DISPLAY_COPIED_TEXT_FOR
    } else {
        false
    };

    let description_text = if display_copied_text {
        Text::raw("copied!").fg(resources.accent_color)
    } else {
        Text::raw("press `c` to copy").fg(resources.main_color)
    }
    .alignment(Alignment::Center);

    let [qr_code_area, address_with_description_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(area);

    let [address_area, description_area] = Layout::vertical([
        Constraint::Length(address_text.height() as u16),
        Constraint::Length(description_text.height() as u16),
    ])
    .flex(Flex::Center)
    .areas(address_with_description_area);

    frame.render_widget(address_text, address_area);
    frame.render_widget(description_text, description_area);

    if model.show_navigation_help {
        let mapping = super::controller::InputEvent::get_mapping();
        common::render_navigation_help(mapping, frame, resources);
    }
}
