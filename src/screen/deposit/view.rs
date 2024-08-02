use std::time::Duration;

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

const DISPLAY_COPIED_TEXT_FOR: Duration = Duration::from_secs(2);

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

    let display_copied_text = if let Some(last_copy) = model.last_address_copy {
        last_copy.elapsed() <= DISPLAY_COPIED_TEXT_FOR
    } else {
        false
    };

    // TODO: Make `copied` label a different color.
    let description = if display_copied_text {
        "copied!"
    } else {
        "press `c` to copy"
    };

    let qr_code = QrCodeWidget::new(pubkey, description.to_string()).with_size(QrCodeSize::Small);

    frame.render_widget(qr_code, area);
}

// TODO: Make it contain only QR-code.
struct QrCodeWidget {
    content: String,
    description: String,
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

        const VERTICAL_BLOCK_PADDING: u16 = 2;
        const HORIZONTAL_BLOCK_PADDING: u16 = 4;

        let block = Block::new()
            .borders(Borders::all())
            .border_type(BorderType::Thick)
            .padding(Padding::symmetric(
                HORIZONTAL_BLOCK_PADDING,
                VERTICAL_BLOCK_PADDING,
            ))
            .black()
            .on_white();

        let address_text = format!("{}\n{}", self.content, self.description);
        let address_text = Text::raw(&address_text).alignment(Alignment::Center);

        let expected_block_height = text.height() as u16 + VERTICAL_BLOCK_PADDING * 2 + 2;
        let expected_block_width = text.width() as u16 + HORIZONTAL_BLOCK_PADDING * 2 + 2;

        let areas = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(expected_block_height),
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
            Constraint::Length(expected_block_width),
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
    fn new(content: String, description: String) -> QrCodeWidget {
        QrCodeWidget {
            content,
            description,
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

        colors
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
            })
    }

    fn render_small(&self, code: QrCode) -> String {
        let width = code.width();
        let colors = code.into_colors();
        let height = colors.len() / width;

        let mut cells = vec![];

        let read_color = |x: usize, y: usize| {
            if x >= width || y >= height {
                QrCodeColor::Light
            } else {
                colors[x + y * width]
            }
        };

        for y in 0..height.div_ceil(2) {
            for x in 0..width {
                cells.push([read_color(x, 2 * y), read_color(x, 2 * y + 1)]);
            }
        }

        cells
            .into_iter()
            .enumerate()
            .map(|(idx, cell)| {
                let str = match (cell[0], cell[1]) {
                    (QrCodeColor::Dark, QrCodeColor::Dark) => "█",
                    (QrCodeColor::Dark, QrCodeColor::Light) => "▀",
                    (QrCodeColor::Light, QrCodeColor::Dark) => "▄",
                    (QrCodeColor::Light, QrCodeColor::Light) => " ",
                };

                if (idx + 1) % width == 0 {
                    [str, "\n"].concat()
                } else {
                    str.to_string()
                }
            })
            .fold(String::new(), |mut acc, x| {
                acc.push_str(&x);
                acc
            })
    }
}
