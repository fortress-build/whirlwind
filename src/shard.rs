use hashbrown::HashTable;
use std::{
    future::Future,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError},
    task::Poll,
};

/// A shard in a [`crate::ShardMap`]. Each shard contains a [`hashbrown::HashTable`] of key-value pairs.
pub(crate) struct Shard<K, V> {
    data: RwLock<HashTable<(K, V)>>,
}

impl<K, V> Shard<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashTable::new()),
        }
    }

    pub fn write<'a>(&'a self) -> impl Future<Output = RwLockWriteGuard<'a, HashTable<(K, V)>>> {
        std::future::poll_fn(|_ctx| match self.data.try_write() {
            Ok(guard) => Poll::Ready(guard),
            Err(TryLockError::WouldBlock) => Poll::Pending,
            Err(TryLockError::Poisoned(_)) => panic!("lock poisoned"),
        })
    }

    pub fn read<'a>(&'a self) -> impl Future<Output = RwLockReadGuard<'a, HashTable<(K, V)>>> {
        std::future::poll_fn(|_ctx| match self.data.try_read() {
            Ok(guard) => Poll::Ready(guard),
            Err(TryLockError::WouldBlock) => Poll::Pending,
            Err(TryLockError::Poisoned(_)) => panic!("lock poisoned"),
        })
    }
}

impl<K, V> std::ops::Deref for Shard<K, V> {
    type Target = RwLock<HashTable<(K, V)>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<K, V> std::ops::DerefMut for Shard<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
