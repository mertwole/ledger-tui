use tokio::task::JoinHandle;

pub struct ApiTask<A, R>(Option<ApiTaskInner<A, R>>);

enum ApiTaskInner<A, R> {
    Api(A),
    Task(JoinHandle<(A, R)>),
}

impl<A, R> ApiTask<A, R> {
    pub fn new(api: A) -> Self {
        Self(Some(ApiTaskInner::Api(api)))
    }

    pub async fn try_fetch_value(
        &mut self,
        spawn_task: impl FnOnce(A) -> JoinHandle<(A, R)>,
    ) -> Option<R> {
        let inner = self.0.take().unwrap();

        let (inner, result) = match inner {
            ApiTaskInner::Api(api) => (ApiTaskInner::Task(spawn_task(api)), None),
            ApiTaskInner::Task(task) => {
                if task.is_finished() {
                    let (api, result) = task.await.unwrap();
                    let task = spawn_task(api);
                    (ApiTaskInner::Task(task), Some(result))
                } else {
                    (ApiTaskInner::Task(task), None)
                }
            }
        };

        self.0 = Some(inner);

        result
    }

    pub async fn abort(self) -> A {
        match self.0.unwrap() {
            ApiTaskInner::Api(api) => api,
            ApiTaskInner::Task(task) => task.await.unwrap().0,
        }
    }
}
