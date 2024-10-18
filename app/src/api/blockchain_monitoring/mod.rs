use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use api_proc_macro::implement_cache;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use tokio::sync::Mutex;

use super::common_types::{Account, Network};

mod bitcoin;
mod ethereum;

// TODO: This API will be fallible (return `Result<...>`) in future.
implement_cache! {
    #[async_trait]
    pub trait BlockchainMonitoringApiT: Send + Sync + 'static {
        // TODO: Decimal is too small for this purpose.
        async fn get_balance(&self, network: Network, account: &Account) -> BigDecimal;

        async fn get_transactions(&self, network: Network, account: &Account) -> Vec<TransactionUid>;

        async fn get_transaction_info(&self, network: Network, tx_uid: &TransactionUid) -> TransactionInfo;
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TransactionUid {
    // TODO: Make private.
    pub uid: String,
}

#[derive(Clone, Debug)]
pub struct TransactionInfo {
    pub ty: TransactionType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub enum TransactionType {
    Deposit { from: Account, amount: Decimal },
    Withdraw { to: Account, amount: Decimal },
}

pub struct BlockchainMonitoringApi {
    network_apis: Mutex<HashMap<Network, Arc<Box<dyn NetworkApi>>>>,
    config: Config,
}

pub struct Config {
    pub network_configs: HashMap<Network, NetworkApiConfig>,
}

#[derive(Clone, Deserialize)]
pub struct NetworkApiConfig {
    pub endpoint: String,
}

impl BlockchainMonitoringApi {
    pub async fn new(config: Config) -> Self {
        Self {
            network_apis: Default::default(),
            config,
        }
    }

    async fn get_or_instantiate_network_api(&self, network: Network) -> Arc<Box<dyn NetworkApi>> {
        match self.network_apis.lock().await.entry(network) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let network_config = self.config.network_configs.get(&network).unwrap();
                let api = ethereum::Api::new(network_config.clone());
                let api: Box<dyn NetworkApi> = Box::from(api);
                let api: Arc<Box<dyn NetworkApi>> = Arc::from(api);

                entry.insert(api.clone());

                api
            }
        }
    }
}

#[async_trait]
impl BlockchainMonitoringApiT for BlockchainMonitoringApi {
    async fn get_balance(&self, network: Network, account: &Account) -> BigDecimal {
        let network_api = self.get_or_instantiate_network_api(network).await;
        network_api.get_balance(account).await
    }

    async fn get_transactions(&self, network: Network, account: &Account) -> Vec<TransactionUid> {
        let network_api = self.get_or_instantiate_network_api(network).await;
        network_api.get_transactions(account).await
    }

    async fn get_transaction_info(
        &self,
        network: Network,
        tx_uid: &TransactionUid,
    ) -> TransactionInfo {
        let network_api = self.get_or_instantiate_network_api(network).await;
        network_api.get_transaction_info(tx_uid).await
    }
}

#[async_trait]
trait NetworkApi: Send + Sync + 'static {
    async fn get_balance(&self, account: &Account) -> BigDecimal;

    async fn get_transactions(&self, account: &Account) -> Vec<TransactionUid>;

    async fn get_transaction_info(&self, tx_uid: &TransactionUid) -> TransactionInfo;
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
            let txs =
                vec![(
                TransactionType::Withdraw {
                    to: Account {
                        pk: "0xMOCK_000000000000000000000000000000000000000000000000000000_MOCK"
                            .to_string(),
                    },
                    amount: Decimal::from_u64(10).unwrap(),
                },
                Utc::now(),
            ),
            (
                TransactionType::Deposit {
                    from: Account {
                        pk: "0xMOCK_000000000000000000000000000000000000000000000000000000_MOCK"
                            .to_string(),
                    },
                    amount: Decimal::from_i128_with_scale(12345, 3),
                },
                Utc::now(),
            )];

            let txs = iter::repeat(txs)
                .flatten()
                .take(tx_count)
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

            Self { txs }
        }
    }

    #[async_trait]
    impl BlockchainMonitoringApiT for BlockchainMonitoringApiMock {
        async fn get_balance(&self, _network: Network, _account: &Account) -> BigDecimal {
            BigDecimal::from_u32(102312).expect("Failed to create BigDecimal from u32")
        }

        async fn get_transactions(
            &self,
            _network: Network,
            _account: &Account,
        ) -> Vec<TransactionUid> {
            self.txs.keys().cloned().collect()
        }

        async fn get_transaction_info(
            &self,
            _network: Network,
            tx_uid: &TransactionUid,
        ) -> TransactionInfo {
            self.txs.get(tx_uid).cloned().unwrap()
        }
    }
}
