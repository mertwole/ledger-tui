use std::time::Instant;

use binance_spot_connector_rust::{market, ureq::BinanceHttpClient};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;

pub trait CoinPriceApiT {
    async fn get_price(&self, from: Coin, to: Coin) -> Option<Decimal>;

    async fn get_price_history(&self, from: Coin, to: Coin) -> Option<PriceHistory>;
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

    async fn get_price_history(&self, _from: Coin, _to: Coin) -> Option<PriceHistory> {
        todo!()
    }
}

pub mod cache {
    use std::collections::HashMap;

    use tokio::sync::Mutex;

    use super::{super::cache_utils, *};
    use crate::api::cache_utils::Mode;

    pub struct Cache<C: CoinPriceApiT> {
        api: C,

        get_price: Mutex<HashMap<(Coin, Coin), Option<Decimal>>>,
        get_price_mode: Mutex<Mode>,

        get_price_history: Mutex<HashMap<(Coin, Coin), Option<PriceHistory>>>,
        get_price_history_mode: Mutex<Mode>,
    }

    impl<C: CoinPriceApiT> Cache<C> {
        pub async fn new(api: C) -> Self {
            Self {
                api,

                get_price: Default::default(),
                get_price_mode: Default::default(),

                get_price_history: Default::default(),
                get_price_history_mode: Default::default(),
            }
        }
    }

    impl<C: CoinPriceApiT> CoinPriceApiT for Cache<C> {
        async fn get_price(&self, from: Coin, to: Coin) -> Option<Decimal> {
            let api_result = self.api.get_price(from, to);
            let api_result = Box::pin(api_result);

            let mut cache = self.get_price.lock().await;
            let cache = cache.entry((from, to));

            let mut mode = self.get_price_mode.lock().await;

            cache_utils::use_cache((from, to), cache, api_result, &mut *mode).await
        }

        async fn get_price_history(&self, from: Coin, to: Coin) -> Option<PriceHistory> {
            let api_result = self.api.get_price_history(from, to);
            let api_result = Box::pin(api_result);

            let mut cache = self.get_price_history.lock().await;
            let cache = cache.entry((from, to));

            let mut mode = self.get_price_history_mode.lock().await;

            cache_utils::use_cache((from, to), cache, api_result, &mut *mode).await
        }
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

        async fn get_price_history(&self, from: Coin, to: Coin) -> Option<PriceHistory> {
            const RESULTS: usize = 100;

            let mut price = self.get_price(from, to).await?;
            let price_interval = price
                .checked_div(Decimal::from_usize(2 * RESULTS).unwrap())
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
