use async_trait::async_trait;

#[async_trait]
pub trait StorageApiT: Send + Sync + 'static {}

pub struct StorageApi {}

impl StorageApi {
    pub async fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl StorageApiT for StorageApi {}

pub mod mock {
    use super::*;

    pub struct StorageApiMock {}

    impl StorageApiMock {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[async_trait]
    impl StorageApiT for StorageApiMock {}
}
