use crate::shard_map::ShardMap;

/// A concurrent set based on a [`ShardMap`] with values of `()`.
pub struct ShardSet<T> {
    inner: ShardMap<T, ()>,
}

impl<T> ShardSet<T>
where
    T: Eq + std::hash::Hash + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: ShardMap::new(),
        }
    }

    pub async fn insert(&self, value: T) {
        self.inner.insert(value, ()).await;
    }

    pub async fn contains(&self, value: &T) -> bool {
        self.inner.contains_key(value).await
    }

    pub async fn remove(&self, value: &T) -> bool {
        self.inner.remove(value).await.is_some()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }

    pub async fn clear(&self) {
        self.inner.clear().await;
    }
}
