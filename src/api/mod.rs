pub mod ledger;

pub trait Api {}

pub trait ApiCache<A: Api> {
    async fn init(api: &A) -> Self;
}

pub enum CachedApi<A: Api, C: ApiCache<A>> {
    Init(InitCachedApi<A, C>),
    Uninit(UninitCachedApi<A>),
}

pub struct UninitCachedApi<A: Api> {
    api: A,
}

pub struct InitCachedApi<A: Api, C: ApiCache<A>> {
    api: A,
    cache: C,
}

impl<A: Api, C: ApiCache<A>> CachedApi<A, C> {
    async fn init(self) -> Self {
        match self {
            Self::Uninit(UninitCachedApi { api }) => Self::Init(InitCachedApi {
                cache: C::init(&api).await,
                api,
            }),
            Self::Init(_) => self,
        }
    }
}
