pub mod ledger;

pub trait Api {
    type Cache;

    async fn get_cache(&self) -> Self::Cache;
}

pub enum CachedApi<A: Api<Cache = C>, C> {
    Init(InitCachedApi<A, C>),
    Uninit(UninitCachedApi<A>),
}

pub struct UninitCachedApi<A: Api> {
    api: A,
}

pub struct InitCachedApi<A: Api<Cache = C>, C> {
    api: A,
    cache: C,
}

impl<A: Api<Cache = C>, C> CachedApi<A, C> {
    async fn init(self) -> Self {
        match self {
            Self::Uninit(UninitCachedApi { api }) => Self::Init(InitCachedApi {
                cache: api.get_cache().await,
                api,
            }),
            Self::Init(_) => self,
        }
    }
}
