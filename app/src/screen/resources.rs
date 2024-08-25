use ratatui::style::Color;

pub struct Resources {
    pub main_color: Color,
    pub secondary_color: Color,
    pub accent_color: Color,
    pub background_color: Color,
    pub qr_code_dark_color: Color,
    pub qr_code_light_color: Color,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            main_color: Color::Black,
            secondary_color: Color::Blue,
            accent_color: Color::Red,
            background_color: Color::White,
            qr_code_dark_color: Color::Black,
            qr_code_light_color: Color::White,
        }
    }
}
