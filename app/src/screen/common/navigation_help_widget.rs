use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Alignment, Rect},
    text::Text,
    widgets::Widget,
};

pub struct NavigationHelpWidget {
    key_bindings: Vec<(KeyCode, String)>,
}

impl NavigationHelpWidget {
    pub fn new(key_bindings: Vec<(KeyCode, String)>) -> Self {
        Self { key_bindings }
    }

    pub fn height(&self) -> usize {
        self.key_bindings.len()
    }

    pub fn min_width(&self) -> usize {
        self.key_bindings
            .iter()
            .map(|(key, description)| min_line_length(&description, &key_name(key)))
            .max()
            .unwrap_or_default()
    }
}

impl Widget for NavigationHelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let width = area.width as usize;

        let text: String = self
            .key_bindings
            .iter()
            .map(|(key, description)| {
                let key_name = key_name(key);
                let line_len = min_line_length(&description, &key_name);

                let padding = width - line_len.min(width);
                let padding_str: String = vec![".".to_string(); padding + 1].into_iter().collect();

                format!("[{}]{}{}", key_name, padding_str, description)
            })
            .intersperse("\n".to_string())
            .collect();

        let text = Text::raw(text).alignment(Alignment::Center);
        text.render(area, buf);
    }
}

fn key_name(key: &KeyCode) -> String {
    match key {
        KeyCode::Char(ch) => ch.to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Enter => "⏎".to_string(),
        _ => unimplemented!(),
    }
}

fn min_line_length(description: &str, key_name: &str) -> usize {
    const BRACKETS_LEN: usize = 2;
    const MINIMAL_SPACING: usize = 1;

    Text::raw(description).width() + Text::raw(key_name).width() + BRACKETS_LEN + MINIMAL_SPACING
}
