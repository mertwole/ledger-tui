pub mod coin_price;
pub mod ledger;

mod cache_utils {
    use std::{future::Future, pin::Pin};

    pub(super) async fn transparent_mode<In, Out>(
        _request: In,
        _cache: Option<&Out>,
        api_result: Pin<Box<impl Future<Output = Out>>>,
    ) -> Out {
        api_result.await
    }
}
