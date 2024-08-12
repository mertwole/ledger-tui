use std::time::Instant;

use api_proc_macro::implement_cache;
use binance_spot_connector_rust::{
    market::{self, klines::KlineInterval},
    ureq::BinanceHttpClient,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;

implement_cache! {
    pub trait CoinPriceApiT {
        async fn get_price(&self, from: Coin, to: Coin) -> Option<Decimal>;

        async fn get_price_history(&self, from: Coin, to: Coin, interval: TimePeriod) -> Option<PriceHistory>;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimePeriod {
    Day,
    Week,
    Month,
    Year,
    All,
}

pub type PriceHistory = Vec<(Instant, Decimal)>;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Coin {
    BTC,
    ETH,
    USDT,
}

impl Coin {
    fn to_api_string(self) -> String {
        match self {
            Self::BTC => "BTC".to_string(),
            Self::ETH => "ETH".to_string(),
            Self::USDT => "USDT".to_string(),
        }
    }
}

pub struct CoinPriceApi {
    client: BinanceHttpClient,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BinanceApiMarketAvgPriceResponse {
    mins: u32,
    price: Decimal,
    close_time: u64,
}

#[derive(Deserialize, Debug)]
#[serde(from = "BinanceApiKlineSerde")]
struct BinanceApiKline {
    open_time: DateTime<Utc>,
    open_price: Decimal,
    high_price: Decimal,
    low_price: Decimal,
    close_price: Decimal,
    volume: Decimal,
    close_time: DateTime<Utc>,
    quote_asset_volume: Decimal,
    number_of_trades: u32,
    taker_buy_base_asset_volume: Decimal,
    taker_buy_quote_asset_volume: Decimal,
    unused_field: String,
}

#[derive(Deserialize)]
struct BinanceApiKlineSerde(
    #[serde(with = "chrono::serde::ts_milliseconds")] DateTime<Utc>,
    Decimal,
    Decimal,
    Decimal,
    Decimal,
    Decimal,
    #[serde(with = "chrono::serde::ts_milliseconds")] DateTime<Utc>,
    Decimal,
    u32,
    Decimal,
    Decimal,
    String,
);

impl From<BinanceApiKlineSerde> for BinanceApiKline {
    fn from(value: BinanceApiKlineSerde) -> Self {
        BinanceApiKline {
            open_time: value.0,
            open_price: value.1,
            high_price: value.2,
            low_price: value.3,
            close_price: value.4,
            volume: value.5,
            close_time: value.6,
            quote_asset_volume: value.7,
            number_of_trades: value.8,
            taker_buy_base_asset_volume: value.9,
            taker_buy_quote_asset_volume: value.10,
            unused_field: value.11,
        }
    }
}

impl CoinPriceApi {
    pub fn new(url: &str) -> Self {
        let client = BinanceHttpClient::with_url(url);
        CoinPriceApi { client }
    }
}

impl CoinPriceApiT for CoinPriceApi {
    async fn get_price(&self, from: Coin, to: Coin) -> Option<Decimal> {
        let pair = [from.to_api_string(), to.to_api_string()].concat();
        let price = self
            .client
            .send(market::avg_price(&pair))
            .unwrap()
            .into_body_str()
            .unwrap();

        let price: BinanceApiMarketAvgPriceResponse = serde_json::from_str(&price).unwrap();

        Some(price.price)
    }

    async fn get_price_history(
        &self,
        from: Coin,
        to: Coin,
        _interval: TimePeriod,
    ) -> Option<PriceHistory> {
        let pair = [from.to_api_string(), to.to_api_string()].concat();
        let history = self
            .client
            .send(market::klines(&pair, KlineInterval::Minutes1))
            .unwrap()
            .into_body_str()
            .unwrap();

        let _history: Vec<BinanceApiKline> = serde_json::from_str(&history).unwrap();

        todo!()
    }
}

pub mod mock {
    use std::{collections::HashMap, time::Duration};

    use rust_decimal::prelude::FromPrimitive;

    use super::*;

    pub struct CoinPriceApiMock {
        prices: HashMap<(Coin, Coin), Decimal>,
    }

    impl CoinPriceApiMock {
        pub fn new() -> Self {
            let mut prices = HashMap::new();

            prices.insert((Coin::BTC, Coin::USDT), dec!(123000.023));
            prices.insert((Coin::ETH, Coin::USDT), dec!(4203.908));
            prices.insert((Coin::USDT, Coin::USDT), dec!(1));

            Self { prices }
        }
    }

    impl CoinPriceApiT for CoinPriceApiMock {
        async fn get_price(&self, from: Coin, to: Coin) -> Option<Decimal> {
            self.prices.get(&(from, to)).cloned()
        }

        async fn get_price_history(
            &self,
            from: Coin,
            to: Coin,
            interval: TimePeriod,
        ) -> Option<PriceHistory> {
            const RESULTS: usize = 100;

            let line_angle = match interval {
                TimePeriod::Day => 2,
                TimePeriod::Week => 3,
                TimePeriod::Month => 4,
                TimePeriod::Year => 5,
                TimePeriod::All => 6,
            };

            let mut price = self.get_price(from, to).await?;
            let price_interval = price
                .checked_div(Decimal::from_usize(line_angle * RESULTS).unwrap())
                .unwrap();

            let mut time = Instant::now();
            let time_interval = Duration::new(10, 0);

            let mut prices = vec![];

            for _ in 0..RESULTS {
                prices.push((time, price));

                price = price.saturating_sub(price_interval);
                time -= time_interval;
            }

            Some(prices)
        }
    }
}
