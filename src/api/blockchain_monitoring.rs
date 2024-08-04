use rust_decimal::Decimal;

use super::common::{Account, Network};
use crate::impl_cache_for_api;

impl_cache_for_api! {
    pub trait BlockchainMonitoringApiT {
        // TODO: Pass `Account` as a reference.
        async fn get_balance(&self, network: Network, account: Account) -> Decimal;
    }
}

pub struct BlockchainMonitoringApi {}

impl BlockchainMonitoringApi {
    pub async fn new() -> Self {
        Self {}
    }
}

impl BlockchainMonitoringApiT for BlockchainMonitoringApi {
    async fn get_balance(&self, _network: Network, _account: Account) -> Decimal {
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
        async fn get_balance(&self, _network: Network, _account: Account) -> Decimal {
            todo!()
        }
    }
}
