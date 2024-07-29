use qrcode::{Color as QrCodeColor, QrCode};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::Buffer,
    style::Stylize,
    text::Text,
    widgets::{Block, BorderType, Borders, Padding, Widget},
    Frame,
};

use super::Model;

pub(super) fn render(model: &Model, frame: &mut Frame<'_>) {
    let state = model
        .state
        .as_ref()
        .expect("Construct should be called at the start of window lifetime");

    let pubkey = state
        .selected_account
        .as_ref()
        .expect("Selected account should be present in state") // TODO: Enforce this rule at `app` level?
        .1
        .get_info()
        .pk;

    let area = frame.size();

    let qr_code = QrCodeWidget::new(pubkey).with_size(QrCodeSize::Small);

    frame.render_widget(qr_code, area);
}

struct QrCodeWidget {
    content: String,
    size: QrCodeSize,
}

enum QrCodeSize {
    Big,
    Small,
}

impl Widget for QrCodeWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let code = QrCode::new(&self.content).unwrap();

        let text = match self.size {
            QrCodeSize::Big => self.render_big(code),
            QrCodeSize::Small => self.render_small(code),
        };

        let text = Text::raw(text).alignment(Alignment::Center);

        let block = Block::new()
            .borders(Borders::all())
            .border_type(BorderType::Thick)
            .padding(Padding::uniform(2))
            .black()
            .on_white();

        let address_text = Text::raw(&self.content).alignment(Alignment::Center);

        let areas = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(text.height() as u16 + 6),
            Constraint::Fill(1),
        ])
        .split(area);

        let footer_area = areas[2];

        let address_area = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(address_text.height() as u16),
            Constraint::Fill(1),
        ])
        .split(footer_area)[1];

        let areas = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length((text.height() as u16 + 6) * 2),
            Constraint::Fill(1),
        ])
        .split(areas[1]);

        let block_area = areas[1];

        let text_area = block.inner(block_area);
        block.render(block_area, buf);
        text.render(text_area, buf);

        address_text.render(address_area, buf);
    }
}

impl QrCodeWidget {
    fn new(content: String) -> QrCodeWidget {
        QrCodeWidget {
            content,
            size: QrCodeSize::Big,
        }
    }

    fn with_size(mut self, size: QrCodeSize) -> Self {
        self.size = size;
        self
    }

    fn render_big(&self, code: QrCode) -> String {
        let width = code.width();
        let colors = code.into_colors();

        let text = colors
            .into_iter()
            .enumerate()
            .map(|(idx, color)| {
                let cell = match color {
                    QrCodeColor::Dark => "██",
                    QrCodeColor::Light => "  ",
                };

                if (idx + 1) % width == 0 {
                    [cell, "\n"].concat()
                } else {
                    cell.to_string()
                }
            })
            .fold(String::new(), |mut acc, x| {
                acc.push_str(&x);
                acc
            });

        text
    }

    fn render_small(&self, code: QrCode) -> String {
        let width = code.width();
        let colors = code.into_colors();
        let height = colors.len() / width;

        const WIDTH_SCALE: usize = 1;
        const HEIGHT_SCALE: usize = 2;

        let cells_width = if width % WIDTH_SCALE == 0 {
            width / WIDTH_SCALE
        } else {
            width / WIDTH_SCALE + 1
        };
        let cells_height = if height % HEIGHT_SCALE == 0 {
            height / HEIGHT_SCALE
        } else {
            height / HEIGHT_SCALE + 1
        };

        let mut cells = vec![];

        for cell_y in 0..cells_height {
            for cell_x in 0..cells_width {
                let mut cell_colors: Vec<QrCodeColor> = vec![];
                for x in cell_x * WIDTH_SCALE..(cell_x + 1) * WIDTH_SCALE {
                    for y in cell_y * HEIGHT_SCALE..(cell_y + 1) * HEIGHT_SCALE {
                        let idx = x + y * width;
                        let color = if idx >= colors.len() {
                            QrCodeColor::Light
                        } else {
                            colors[idx]
                        };
                        cell_colors.push(color);
                    }
                }
                cells.push(cell_colors);
            }
        }

        let text = cells
            .into_iter()
            .enumerate()
            .map(|(idx, cell)| {
                let str = match (cell[0], cell[1]) {
                    (QrCodeColor::Dark, QrCodeColor::Dark) => "█",
                    (QrCodeColor::Dark, QrCodeColor::Light) => "▀",
                    (QrCodeColor::Light, QrCodeColor::Dark) => "▄",
                    (QrCodeColor::Light, QrCodeColor::Light) => " ",
                };

                if (idx + 1) % cells_width == 0 {
                    [str, "\n"].concat()
                } else {
                    str.to_string()
                }
            })
            .fold(String::new(), |mut acc, x| {
                acc.push_str(&x);
                acc
            });

        text
    }
}
