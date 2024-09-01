use ratatui::{
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::Widget,
};

pub struct NavigationHelpWidget {
    key_bindings: Vec<(KeyCode, String)>,
}

impl NavigationHelpWidget {
    pub fn new(key_bindings: Vec<(KeyCode, String)>) -> Self {
        Self { key_bindings }
    }
}

impl Widget for NavigationHelpWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        todo!()
    }
}
