use std::{
    collections::{HashMap, hash_map::Entry},
    future::Future,
    hash::Hash,
    pin::Pin,
    time::{Duration, Instant},
};
use tokio::time::sleep;

#[derive(Default, Clone, Copy)]
pub enum ModePlan {
    #[default]
    Transparent,
    TimedOut(Duration),
    Slow(Duration),
}

impl ModePlan {
    pub fn into_mode<In: Hash + PartialEq + Eq, Out>(self) -> Mode<In, Out> {
        match self {
            Self::Transparent => Mode::new_transparent(),
            Self::TimedOut(timeout) => Mode::new_timed_out(timeout),
            Self::Slow(delay) => Mode::new_slow(delay),
        }
    }
}

#[derive(Default, Clone)]
pub enum Mode<In: Hash + PartialEq + Eq, Out> {
    /// This type of cache will call API each time the corresponding method is called.
    #[default]
    Transparent,
    /// This type of cache will call API only if some specified time have passed after previous call.
    /// It will return value from cache elsewhere.
    TimedOut(TimedOutMode<In, Out>),
    /// This type of cache will delay calls to API to simulate network or i/o delays.
    Slow(Duration),
}

#[derive(Clone)]
pub struct TimedOutMode<In, Out> {
    timeout: Duration,
    previous_request: HashMap<In, Instant>,
    cache: HashMap<In, Out>,
}

impl<In: Hash + PartialEq + Eq, Out> Mode<In, Out> {
    pub fn new_transparent() -> Self {
        Self::Transparent
    }

    pub fn new_timed_out(timeout: Duration) -> Self {
        Self::TimedOut(TimedOutMode {
            timeout,
            previous_request: Default::default(),
            cache: Default::default(),
        })
    }

    pub fn new_slow(delay: Duration) -> Self {
        Self::Slow(delay)
    }
}

pub(super) async fn use_cache<In, Out>(
    request: In,
    api_result: Pin<Box<impl Future<Output = Out>>>,
    mode: &mut Mode<In, Out>,
) -> Out
where
    Out: Clone,
    In: Hash + PartialEq + Eq + Clone,
{
    match mode {
        Mode::Transparent => transparent_mode(request, api_result).await,
        Mode::TimedOut(state) => timed_out_mode(request, api_result, state).await,
        Mode::Slow(delay) => slow_mode(request, api_result, *delay).await,
    }
}

async fn transparent_mode<In, Out>(
    _request: In,
    api_result: Pin<Box<impl Future<Output = Out>>>,
) -> Out {
    api_result.await
}

async fn timed_out_mode<In, Out>(
    request: In,
    api_result: Pin<Box<impl Future<Output = Out>>>,
    state: &mut TimedOutMode<In, Out>,
) -> Out
where
    Out: Clone,
    In: Hash + PartialEq + Eq + Clone,
{
    if let Some(cache) = state.cache.get(&request) {
        let previous_request_entry = state.previous_request.entry(request.clone());
        if let Entry::Occupied(previous_request) = previous_request_entry {
            if previous_request.get().elapsed() < state.timeout {
                return cache.clone();
            }
        }
    }

    let result = api_result.await;

    state.cache.insert(request.clone(), result.clone());
    state.previous_request.insert(request, Instant::now());

    result
}

async fn slow_mode<In, Out>(
    _request: In,
    api_result: Pin<Box<impl Future<Output = Out>>>,
    delay: Duration,
) -> Out {
    sleep(delay).await;

    api_result.await
}
