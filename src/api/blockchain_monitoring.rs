use std::time::Instant;

use rust_decimal::Decimal;

use super::common::{Account, Network};
use crate::impl_cache_for_api;

impl_cache_for_api! {
    pub trait BlockchainMonitoringApiT {
        // TODO: Pass `Account` as a reference.
        async fn get_balance(&self, network: Network, account: Account) -> Decimal;

        async fn get_transactions(&self, network: Network, account: Account) -> Vec<TransactionUid>;

        // TODO: Pass `TransactionUid` as a reference.
        async fn get_transaction_info(&self, network: Network, tx_uid: TransactionUid) -> TransactionInfo;
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TransactionUid {
    uid: String,
}

#[derive(Clone)]
pub struct TransactionInfo {
    ty: TransactionType,
    timestamp: Instant,
}

#[derive(Clone)]
pub enum TransactionType {
    Deposit { from: Account, amount: Decimal },
    Withdraw { to: Account, amount: Decimal },
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

    async fn get_transactions(&self, _network: Network, _account: Account) -> Vec<TransactionUid> {
        todo!()
    }

    async fn get_transaction_info(
        &self,
        _network: Network,
        _tx_uid: TransactionUid,
    ) -> TransactionInfo {
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

        async fn get_transactions(
            &self,
            _network: Network,
            _account: Account,
        ) -> Vec<TransactionUid> {
            todo!()
        }

        async fn get_transaction_info(
            &self,
            _network: Network,
            _tx_uid: TransactionUid,
        ) -> TransactionInfo {
            todo!()
        }
    }
}
