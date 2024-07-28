use futures::executor::block_on;
use qrcode::{Color as QrCodeColor, QrCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Buffer,
    style::{Style, Stylize},
    symbols,
    text::Text,
    widgets::{canvas::Label, Axis, Block, Borders, Chart, Dataset, GraphType, Widget},
    Frame,
};

use crate::api::{
    coin_price::{Coin, CoinPriceApiT},
    ledger::{LedgerApiT, Network},
};

use super::Model;

pub(super) fn render<L: LedgerApiT, C: CoinPriceApiT>(model: &Model<L, C>, frame: &mut Frame<'_>) {
    let _state = model
        .state
        .as_ref()
        .expect("Construct should be called at the start of window lifetime");

    let area = frame.size();

    let data =
        hex::decode("0123456789012345678901234567890101234567890123456789012345678901").unwrap();
    let qr_code = QrCodeWidget::new(data).with_size(QrCodeSize::Half);

    frame.render_widget(qr_code, area);
}

struct QrCodeWidget {
    content: Vec<u8>,
    size: QrCodeSize,
}

enum QrCodeSize {
    // 2 * 4
    Full,
    // 1 * 2
    Half,
    // 0.5 x 1
    Small,
}

impl Widget for QrCodeWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let text = self.render_small();
        let text = Text::raw(text).alignment(Alignment::Center);

        let area = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(text.height() as u16),
            Constraint::Fill(1),
        ])
        .split(area)[1];

        text.render(area, buf);
    }
}

impl QrCodeWidget {
    fn new(content: Vec<u8>) -> QrCodeWidget {
        QrCodeWidget {
            content,
            size: QrCodeSize::Full,
        }
    }

    fn with_size(mut self, size: QrCodeSize) -> Self {
        self.size = size;
        self
    }

    fn render_small(&self) -> String {
        let data = bs58::encode(&self.content).into_vec();
        let code = QrCode::new(data).unwrap();

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

// 38 * 38
// ⊠ ■ ▣ ▩ ◼ ⬛ █
// ▀ ▁ ▂ ▃ ▄ ▅ ▆ ▇ █ ▉ ▊ ▋ ▌  ▍ ▍

// █ █ █ █ █ █ █ █ █ █ █ █ █ █ █ █
// ▖ ▗ ▘ ▙ ▚ ▛ ▜ ▝ ▞ ▟ ▐ ▌ ▄ ▀
