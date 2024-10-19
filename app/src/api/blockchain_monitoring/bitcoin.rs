use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};

use async_trait::async_trait;
use bigdecimal::{BigDecimal, Zero};

use super::{Account, NetworkApi, NetworkApiConfig, TransactionInfo, TransactionUid};

pub struct Api {
    _client: HttpClient,
}

impl Api {
    pub fn new(config: NetworkApiConfig) -> Api {
        let client = HttpClientBuilder::new().build(&config.endpoint).unwrap();

        Api { _client: client }
    }
}

// TODO: Bitcoin core API is very linmited on what we can request, so probably
// such an api possible only for using in pair with some kind of indexer.
#[async_trait]
impl NetworkApi for Api {
    async fn get_balance(&self, _account: &Account) -> BigDecimal {
        BigDecimal::zero()
    }

    async fn get_transactions(&self, _account: &Account) -> Vec<TransactionUid> {
        vec![]
    }

    async fn get_transaction_info(&self, _tx_uid: &TransactionUid) -> TransactionInfo {
        unimplemented!()
    }
}
