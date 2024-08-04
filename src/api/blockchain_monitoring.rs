use rust_decimal::Decimal;

use crate::impl_cache_for_api;

impl_cache_for_api! {
    pub trait BlockchainMonitoringApiT {
        async fn get_balance(&self, network: Network) -> Decimal;
    }
}

// TODO: Move it level up as it'll be shared between APIs?
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Network {
    Bitcoin,
    Ethereum,
}

pub struct BlockchainMonitoringApi {}

impl BlockchainMonitoringApi {
    pub async fn new() -> Self {
        Self {}
    }
}

impl BlockchainMonitoringApiT for BlockchainMonitoringApi {
    async fn get_balance(&self, network: Network) -> Decimal {
        todo!()
    }
}

pub mod mock {
    use super::*;

    pub struct BlockchainMonitoringApiMock {}

    impl BlockchainMonitoringApiMock {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl BlockchainMonitoringApiT for BlockchainMonitoringApiMock {
        async fn get_balance(&self, network: Network) -> Decimal {
            todo!()
        }
    }
}
