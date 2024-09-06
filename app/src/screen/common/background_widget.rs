use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::Widget,
};

pub struct BackgroundWidget {
    color: Color,
}

impl BackgroundWidget {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Widget for BackgroundWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let line = Line::raw(" ".repeat(area.width as usize)).bg(self.color);
        for y in area.y..area.y + area.height {
            buf.set_line(area.x, y, &line, area.width);
        }
    }
}
