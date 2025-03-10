use input_mapping_common::InputMappingT;
use ratatui::Frame;

use crate::{
    api::{ledger::LedgerApiT, storage::StorageApiT},
    screen::{common, resources::Resources},
};

use super::Model;

pub(super) fn render<L: LedgerApiT, S: StorageApiT>(
    model: &Model<L, S>,
    frame: &mut Frame<'_>,
    resources: &Resources,
) {
    if model.show_navigation_help {
        let mapping = super::controller::InputEvent::get_mapping();
        common::render_navigation_help(mapping, frame, resources);
    }
}
