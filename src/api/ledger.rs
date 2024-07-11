pub trait LedgerApiT {}

pub struct LedgerApi {}

impl LedgerApiT for LedgerApi {}

mod mock {
    use super::*;

    pub struct LedgerApiMock {}

    impl LedgerApiT for LedgerApiMock {}
}
