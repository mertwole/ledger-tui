pub mod coin_price;
pub mod ledger;

mod cache_utils {
    use std::{
        collections::hash_map::Entry,
        future::Future,
        pin::Pin,
        time::{Duration, Instant},
    };

    #[derive(Default)]
    pub enum Mode {
        /// This type of cache will call API each time the corresponding method is called.
        #[default]
        Transparent,
        /// This type of cache will call API only if some specified time have passed after previous call.
        /// It will return value from cache elsewhere.
        TimedOut(TimedOutMode),
    }

    pub struct TimedOutMode {
        timeout: Duration,
        previous_request: Instant,
    }

    impl Mode {
        pub fn new_transparent() -> Self {
            Self::Transparent
        }

        pub fn new_timed_out(timeout: Duration) -> Self {
            let now = Instant::now();

            Self::TimedOut(TimedOutMode {
                timeout,
                previous_request: now - timeout,
            })
        }
    }

    pub(super) async fn use_cache<In, Out>(
        request: In,
        cache: Entry<'_, In, Out>,
        api_result: Pin<Box<impl Future<Output = Out>>>,
        mode: &mut Mode,
    ) -> Out
    where
        Out: Clone,
    {
        match mode {
            Mode::Transparent => transparent_mode(request, cache, api_result).await,
            Mode::TimedOut(state) => timed_out_mode(request, cache, api_result, state).await,
        }
    }

    async fn transparent_mode<In, Out>(
        _request: In,
        _cache: Entry<'_, In, Out>,
        api_result: Pin<Box<impl Future<Output = Out>>>,
    ) -> Out {
        api_result.await
    }

    async fn timed_out_mode<In, Out>(
        _request: In,
        cache: Entry<'_, In, Out>,
        api_result: Pin<Box<impl Future<Output = Out>>>,
        state: &mut TimedOutMode,
    ) -> Out
    where
        Out: Clone,
    {
        if state.previous_request.elapsed() >= state.timeout || matches!(cache, Entry::Vacant(_)) {
            let result = api_result.await;
            state.previous_request = Instant::now();

            match cache {
                Entry::Occupied(mut entry) => {
                    entry.insert(result.clone());
                }
                Entry::Vacant(entry) => {
                    entry.insert(result.clone());
                }
            }

            return result;
        }

        match cache {
            Entry::Occupied(entry) => entry.get().clone(),
            _ => unreachable!("Entry is checked above to be Occupied"),
        }
    }
}
