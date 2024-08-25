use std::time::Duration;

use qrcode::{Color as QrCodeColor, QrCode};
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    prelude::Buffer,
    style::Stylize,
    text::Text,
    widgets::{Block, BorderType, Borders, Padding, Widget},
    Frame,
};

use crate::{
    api::{
        blockchain_monitoring::BlockchainMonitoringApiT, coin_price::CoinPriceApiT,
        ledger::LedgerApiT,
    },
    screen::resources::Resources,
};

use super::Model;

const DISPLAY_COPIED_TEXT_FOR: Duration = Duration::from_secs(2);

pub(super) fn render<L: LedgerApiT, C: CoinPriceApiT, M: BlockchainMonitoringApiT>(
    model: &Model<L, C, M>,
    frame: &mut Frame<'_>,
    _resources: &Resources,
) {
    let pubkey = model
        .state
        .selected_account
        .as_ref()
        .expect("Selected account should be present in state") // TODO: Enforce this rule at `app` level?
        .1
        .get_info()
        .pk;

    let area = frame.size();

    let address_text = Text::raw(&pubkey).alignment(Alignment::Center);

    let display_copied_text = if let Some(last_copy) = model.last_address_copy {
        last_copy.elapsed() <= DISPLAY_COPIED_TEXT_FOR
    } else {
        false
    };

    let description_text = if display_copied_text {
        Text::raw("copied!").green()
    } else {
        Text::raw("press `c` to copy")
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

    let qr_code = QrCodeWidget::new(pubkey.clone()).with_size(QrCodeSize::Small);

    frame.render_widget(qr_code, qr_code_area);
    frame.render_widget(address_text, address_area);
    frame.render_widget(description_text, description_area);
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
        let code = match self.size {
            QrCodeSize::Big => self.render_big(code),
            QrCodeSize::Small => self.render_small(code),
        };
        let code = Text::raw(code).alignment(Alignment::Center);

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

        let expected_block_height = code.height() as u16 + VERTICAL_BLOCK_PADDING * 2 + 2;
        let expected_block_width = code.width() as u16 + HORIZONTAL_BLOCK_PADDING * 2 + 2;

        let [area] = Layout::horizontal([Constraint::Length(expected_block_width)])
            .flex(Flex::Center)
            .areas(area);
        let [area] = Layout::vertical([Constraint::Length(expected_block_height)])
            .flex(Flex::Center)
            .areas(area);

        let code_area = block.inner(area);

        block.render(area, buf);
        code.render(code_area, buf);
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
