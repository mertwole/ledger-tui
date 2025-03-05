use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;

#[async_trait]
#[allow(dead_code)]
pub trait StorageApiT: Send + Sync + 'static {
    async fn save(&mut self, name: &str, content: String);

    async fn load(&mut self, name: &str) -> Option<String>;
}

pub struct StorageApi {
    path: PathBuf,
    cache: HashMap<String, String>,
}

impl StorageApi {
    pub fn new(path: PathBuf) -> Self {
        if !path.is_dir() {
            panic!("Expected directory");
        }

        Self {
            path,
            cache: HashMap::new(),
        }
    }
}

#[async_trait]
impl StorageApiT for StorageApi {
    async fn save(&mut self, name: &str, content: String) {
        tokio::fs::write(self.path.join(name), content.clone())
            .await
            .unwrap();

        self.cache.insert(name.to_string(), content);
    }

    async fn load(&mut self, name: &str) -> Option<String> {
        if let Some(cached) = self.cache.get(name) {
            return Some(cached.clone());
        }

        let path = self.path.join(name);

        let file_exists = tokio::fs::try_exists(&path).await;
        if !matches!(file_exists, Ok(true)) {
            return None;
        }

        let content = tokio::fs::read_to_string(path).await.unwrap();

        self.cache.insert(name.to_string(), content.clone());

        Some(content)
    }
}

pub mod mock {
    use super::*;

    pub struct StorageApiMock {
        storage: HashMap<String, String>,
    }

    impl StorageApiMock {
        pub fn new() -> Self {
            Self {
                storage: HashMap::new(),
            }
        }
    }

    #[async_trait]
    impl StorageApiT for StorageApiMock {
        async fn save(&mut self, name: &str, content: String) {
            self.storage.insert(name.to_string(), content);
        }

        async fn load(&mut self, name: &str) -> Option<String> {
            self.storage.get(name).cloned()
        }
    }
}
