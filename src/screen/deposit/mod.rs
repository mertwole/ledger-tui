use ratatui::Frame;

use super::{OutgoingMessage, Screen};
use crate::app::StateRegistry;

mod controller;
mod view;

pub struct Model {
    state: Option<StateRegistry>,
}

impl Model {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl Screen for Model {
    fn construct(&mut self, state: StateRegistry) {
        self.state = Some(state);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        view::render(self, frame);
    }

    fn tick(&mut self) -> Option<OutgoingMessage> {
        controller::process_input()
    }

    fn deconstruct(self: Box<Self>) -> StateRegistry {
        self.state
            .expect("Construct should be called at the start of window lifetime")
    }
}
