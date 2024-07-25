use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub trait CoinPriceApiT {
    async fn get_price(&self, from: Coin, to: Coin) -> Option<Decimal>;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Coin {
    BTC,
    ETH,
    USDT,
}

pub struct CoinPriceApi {}

impl CoinPriceApiT for CoinPriceApi {
    async fn get_price(&self, _from: Coin, _to: Coin) -> Option<Decimal> {
        todo!()
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
