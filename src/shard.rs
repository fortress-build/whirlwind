use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub(crate) type Inner<K, V> = hashbrown::HashTable<(K, V)>;
pub(crate) type ShardReader<'a, K, V> = RwLockReadGuard<'a, Inner<K, V>>;
pub(crate) type ShardWriter<'a, K, V> = RwLockWriteGuard<'a, Inner<K, V>>;

/// A shard in a [`crate::ShardMap`]. Each shard contains a [`hashbrown::HashTable`] of key-value pairs.
pub(crate) struct Shard<K, V> {
    data: RwLock<Inner<K, V>>,
}

impl<K, V> Shard<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: RwLock::new(Inner::with_capacity(capacity)),
        }
    }

    pub async fn write<'a>(&'a self) -> ShardWriter<'a, K, V> {
        self.data.write().await
    }

    pub async fn read<'a>(&'a self) -> ShardReader<'a, K, V> {
        self.data.read().await
    }
}

impl<K, V> std::ops::Deref for Shard<K, V> {
    type Target = RwLock<Inner<K, V>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<K, V> std::ops::DerefMut for Shard<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
