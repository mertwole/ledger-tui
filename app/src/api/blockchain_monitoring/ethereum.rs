use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder, RootProvider},
    transports::http::{Client, Http},
};
use async_trait::async_trait;
use bigdecimal::{BigDecimal, One};
use rust_decimal::prelude::{FromPrimitive, Zero};

use super::{Account, NetworkApi, NetworkApiConfig, TransactionInfo, TransactionUid};

const WEI_IN_ETH: u64 = 1_000_000_000_000_000_000;

pub struct Api {
    provider: RootProvider<Http<Client>>,
}

impl Api {
    pub fn new(config: NetworkApiConfig) -> Api {
        let rpc_url = config.endpoint.parse().unwrap();
        let provider = ProviderBuilder::new().on_http(rpc_url);

        Api { provider }
    }
}

#[async_trait]
impl NetworkApi for Api {
    async fn get_balance(&self, account: &Account) -> BigDecimal {
        let account = Address::parse_checksummed(&account.pk, None).unwrap();
        let balance = self.provider.get_balance(account).await.unwrap();
        let balance_le: [u8; 32] = balance.to_le_bytes();

        let mut balance = BigDecimal::zero();
        let mut exp = BigDecimal::one();
        for byte in balance_le {
            balance += exp.clone()
                * BigDecimal::from_u8(byte).expect("Failed to create BigDecimal from u8");
            exp *= BigDecimal::from_usize(1 << 8).expect("Failed to create BigDecimal from usize");
        }

        balance / BigDecimal::from_u64(WEI_IN_ETH).expect("Failed to create BigDecimal from u64")
    }

    async fn get_transactions(&self, _account: &Account) -> Vec<TransactionUid> {
        vec![]
    }

    async fn get_transaction_info(&self, _tx_uid: &TransactionUid) -> TransactionInfo {
        todo!()
    }
}
