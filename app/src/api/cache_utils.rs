use std::{
    collections::{hash_map::Entry, HashMap},
    future::Future,
    hash::Hash,
    pin::Pin,
    time::{Duration, Instant},
};

#[derive(Default, Clone, Copy)]
pub enum ModePlan {
    #[default]
    Transparent,
    TimedOut(Duration),
}

impl ModePlan {
    pub fn into_mode<In: Hash + PartialEq + Eq>(self) -> Mode<In> {
        match self {
            Self::Transparent => Mode::new_transparent(),
            Self::TimedOut(timeout) => Mode::new_timed_out(timeout),
        }
    }
}

#[derive(Default, Clone)]
pub enum Mode<In: Hash + PartialEq + Eq> {
    /// This type of cache will call API each time the corresponding method is called.
    #[default]
    Transparent,
    /// This type of cache will call API only if some specified time have passed after previous call.
    /// It will return value from cache elsewhere.
    TimedOut(TimedOutMode<In>),
}

#[derive(Clone)]
pub struct TimedOutMode<In> {
    timeout: Duration,
    previous_request: HashMap<In, Instant>,
}

impl<In: Hash + PartialEq + Eq> Mode<In> {
    pub fn new_transparent() -> Self {
        Self::Transparent
    }

    pub fn new_timed_out(timeout: Duration) -> Self {
        Self::TimedOut(TimedOutMode {
            timeout,
            previous_request: Default::default(),
        })
    }
}

pub(super) async fn use_cache<In, Out>(
    request: In,
    cache: Entry<'_, In, Out>,
    api_result: Pin<Box<impl Future<Output = Out>>>,
    mode: &mut Mode<In>,
) -> Out
where
    Out: Clone,
    In: Hash + PartialEq + Eq + Clone,
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
    request: In,
    cache: Entry<'_, In, Out>,
    api_result: Pin<Box<impl Future<Output = Out>>>,
    state: &mut TimedOutMode<In>,
) -> Out
where
    Out: Clone,
    In: Hash + PartialEq + Eq + Clone,
{
    if let Entry::Occupied(cache) = &cache {
        let previous_request_entry = state.previous_request.entry(request.clone());
        if let Entry::Occupied(previous_request) = previous_request_entry {
            if previous_request.get().elapsed() < state.timeout {
                return cache.get().clone();
            }
        }
    }

    let result = api_result.await;
    cache.insert_entry(result.clone());

    state.previous_request.insert(request, Instant::now());

    result
}
