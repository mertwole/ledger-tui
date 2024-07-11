use super::{Api, ApiCache};

pub trait LedgerApiT: Api {}

pub struct LedgerApiCache {}

impl<A: LedgerApiT> ApiCache<A> for LedgerApiCache {
    async fn init(api: &A) -> Self {
        LedgerApiCache {}
    }
}

mod api {
    use super::*;

    pub struct LedgerApi {}

    impl Api for LedgerApi {}

    impl LedgerApiT for LedgerApi {}
}

mod mock {
    use super::*;

    pub struct LedgerApiMock {}

    impl Api for LedgerApiMock {}

    impl LedgerApiT for LedgerApiMock {}
}
