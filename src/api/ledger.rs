use super::Api;

pub trait LedgerApi: Api<Cache = LedgerCache> {
    //
}

pub struct Ledger {
    //
}

pub struct LedgerCache {}

impl Api for Ledger {
    type Cache = LedgerCache;

    async fn get_cache(&self) -> Self::Cache {
        LedgerCache {}
    }
}

impl LedgerApi for Ledger {
    //
}

pub struct LedgerMock {
    //
}

impl Api for LedgerMock {
    type Cache = LedgerCache;

    async fn get_cache(&self) -> Self::Cache {
        LedgerCache {}
    }
}

impl LedgerApi for LedgerMock {
    //
}
