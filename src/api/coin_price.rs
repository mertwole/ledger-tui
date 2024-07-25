use binance_spot_connector_rust::{market, ureq::BinanceHttpClient};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;

pub trait CoinPriceApiT {
    async fn get_price(&self, from: Coin, to: Coin) -> Option<Decimal>;
}

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct BinanceApiMarketAvgPriceResponse {
    mins: u32,
    price: Decimal,
    close_time: u64,
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
}

pub mod mock {
    use std::collections::HashMap;

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
    }
}
