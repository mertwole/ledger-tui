use std::time::Duration;

use futures::executor::block_on;
use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use crate::{
    api::{
        coin_price::{Coin, CoinPriceApiT},
        ledger::{LedgerApiT, Network},
    },
    app::StateRegistry,
};

use super::{EventExt, OutgoingMessage, Window};

pub struct Asset<L: LedgerApiT, C: CoinPriceApiT> {
    _ledger_api: L,
    coin_price_api: C,

    state: Option<StateRegistry>,
}

impl<L: LedgerApiT, C: CoinPriceApiT> Window for Asset<L, C> {
    fn construct(&mut self, state: StateRegistry) {
        self.state = Some(state);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let state = self
            .state
            .as_ref()
            .expect("Construct should be called at the start of window lifetime");

        // TODO: Enforce this rule at `app` level?
        let selected_account = state
            .selected_account
            .as_ref()
            .expect("Selected account should be present in state");

        let area = frame.size();

        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)])
            .split(area);

        let price_chart_block = Block::new().title("Price").borders(Borders::all());
        let price_chart_area = price_chart_block.inner(areas[0]);
        frame.render_widget(price_chart_block, areas[0]);
        self.render_price_chart(frame, price_chart_area, &selected_account.0);

        // TODO: Display transactions here.
        let txs_list_block = Block::new().title("Transactions").borders(Borders::all());
        frame.render_widget(txs_list_block, areas[1]);
    }

    fn tick(&mut self) -> Option<OutgoingMessage> {
        self.process_input()
    }

    fn deconstruct(self: Box<Self>) -> StateRegistry {
        self.state
            .expect("Construct should be called at the start of window lifetime")
    }
}

impl<L: LedgerApiT, C: CoinPriceApiT> Asset<L, C> {
    pub fn new(ledger_api: L, coin_price_api: C) -> Self {
        Self {
            _ledger_api: ledger_api,
            coin_price_api,
            state: None,
        }
    }

    fn render_price_chart(&self, frame: &mut Frame<'_>, area: Rect, network: &Network) {
        let coin = match network {
            Network::Bitcoin => Coin::BTC,
            Network::Ethereum => Coin::ETH,
        };

        let mut prices = block_on(self.coin_price_api.get_price_history(coin, Coin::USDT)).unwrap();
        prices.sort_by(|a, b| a.0.cmp(&b.0));

        let mut price_bounds = [f64::MAX, f64::MIN];
        for (_, price) in &prices {
            let price: f64 = price.clone().try_into().unwrap();
            price_bounds[0] = price_bounds[0].min(price);
            price_bounds[1] = price_bounds[1].max(price);
        }

        let price_data: Vec<_> = prices
            .into_iter()
            .enumerate()
            .map(|(idx, price)| (idx as f64, price.1.try_into().unwrap()))
            .collect();

        let datasets = vec![Dataset::default()
            .marker(symbols::Marker::Bar)
            .graph_type(GraphType::Line)
            .style(Style::default().magenta())
            .data(&price_data)];

        let x_axis = Axis::default()
            .style(Style::default().white())
            .bounds([0.0, price_data.len() as f64]);

        let y_axis = Axis::default()
            .style(Style::default().white())
            .bounds(price_bounds);

        let chart = Chart::new(datasets).x_axis(x_axis).y_axis(y_axis);

        frame.render_widget(chart, area);
    }

    fn process_input(&mut self) -> Option<OutgoingMessage> {
        if !event::poll(Duration::ZERO).unwrap() {
            return None;
        }

        let event = event::read().unwrap();

        if event.is_key_pressed(KeyCode::Char('q')) {
            return Some(OutgoingMessage::Exit);
        }

        None
    }
}
