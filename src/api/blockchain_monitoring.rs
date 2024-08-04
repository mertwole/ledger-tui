#![allow(dead_code)] // TODO: Remove

use std::time::Instant;

use rust_decimal::Decimal;

use super::common::{Account, Network};
use crate::impl_cache_for_api;

// TODO: This API will be fallible (return `Result<...>`) in future.
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
    use std::{collections::HashMap, iter};

    use rust_decimal::prelude::FromPrimitive;

    use super::*;

    pub struct BlockchainMonitoringApiMock {
        txs: HashMap<TransactionUid, TransactionInfo>,
    }

    impl BlockchainMonitoringApiMock {
        pub fn new(tx_count: usize) -> Self {
            let txs: Vec<_> = vec![(
                TransactionType::Withdraw {
                    to: Account {
                        pk: "0xMOCK_000000000000000000000000000000000000000000000000000000_MOCK"
                            .to_string(),
                    },
                    amount: Decimal::from_u64(1).unwrap(),
                },
                Instant::now(),
            )]
            .into_iter()
            .enumerate()
            .map(|(idx, (ty, timestamp))| {
                (
                    TransactionUid {
                        uid: format!("MOCK_TX_HASH_{}", idx),
                    },
                    TransactionInfo { ty, timestamp },
                )
            })
            .collect();

            let txs = iter::repeat(txs).flatten().take(tx_count).collect();

            Self { txs }
        }
    }

    impl BlockchainMonitoringApiT for BlockchainMonitoringApiMock {
        async fn get_balance(&self, _network: Network, _account: Account) -> Decimal {
            Decimal::from_i128_with_scale(102312, 1)
        }

        async fn get_transactions(
            &self,
            _network: Network,
            _account: Account,
        ) -> Vec<TransactionUid> {
            self.txs.keys().cloned().collect()
        }

        async fn get_transaction_info(
            &self,
            _network: Network,
            tx_uid: TransactionUid,
        ) -> TransactionInfo {
            self.txs.get(&tx_uid).cloned().unwrap()
        }
    }
}
