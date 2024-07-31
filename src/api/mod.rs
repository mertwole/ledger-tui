pub mod coin_price;
pub mod ledger;

pub mod cache_utils {
    use std::{
        collections::{hash_map::Entry, HashMap},
        future::Future,
        hash::Hash,
        pin::Pin,
        time::{Duration, Instant},
    };

    #[macro_export]
    macro_rules! impl_cache_for_api {
        (
            pub trait $api_trait: ident {
                $(
                    $(#[$($attributes:tt)*])*
                    async fn $method_name:ident(
                        &self,
                        $($arg_name:ident : $arg_type:ty),*
                        $(,)?
                    ) -> $return_type:ty
                );*
                $(;)?
            }
        ) => {
            pub trait $api_trait {
                $(
                    $(#[$($attributes)*])*
                    async fn $method_name(
                        &self,
                        $($arg_name : $arg_type),*
                    ) -> $return_type
                );*;
            }

            pub mod cache {
                use std::{cell::RefCell, collections::HashMap};

                use super::*;
                use crate::api::cache_utils::Mode;

                ::paste::paste! {
                    pub struct Cache<A: super::$api_trait> {
                        api: A,

                        $(
                            $method_name: RefCell<HashMap<(Coin, Coin), $return_type>>,
                            [<__ $method_name _mode>] : RefCell<Mode<(Coin, Coin)>>,
                        )*
                    }


                    impl<A: super::$api_trait> Cache<A> {
                        pub async fn new(api: A) -> Self {
                            Self {
                                api,

                                $(
                                    $method_name: Default::default(),
                                    [<__ $method_name _mode>] : Default::default(),
                                )*
                            }
                        }

                        // TODO: implement wrappers for all fields.
                        pub fn set_get_price_mode(&mut self, mode: Mode<(Coin, Coin)>) {
                            (*self.__get_price_mode.borrow_mut()) = mode;
                        }
                    }

                    impl<A: super::$api_trait> super::$api_trait for Cache<A> {
                        $(
                            $(#[$($attributes)*])*
                            async fn $method_name(
                                &self,
                                $($arg_name : $arg_type),*
                            ) -> $return_type {
                                let api_result = self.api.$method_name($($arg_name),*);
                                let api_result = Box::pin(api_result);

                                let mut cache = self.$method_name.borrow_mut();
                                let cache = cache.entry(($($arg_name),*));

                                let mut mode = self.[<__ $method_name _mode>].borrow_mut();

                                $crate::api::cache_utils::use_cache(($($arg_name),*), cache, api_result, &mut *mode).await
                            }
                        )*
                    }
                }
            }
        };
    }

    #[derive(Default)]
    pub enum Mode<In: Hash + PartialEq + Eq> {
        /// This type of cache will call API each time the corresponding method is called.
        #[default]
        Transparent,
        /// This type of cache will call API only if some specified time have passed after previous call.
        /// It will return value from cache elsewhere.
        TimedOut(TimedOutMode<In>),
    }

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
}
