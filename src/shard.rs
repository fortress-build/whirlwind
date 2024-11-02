use hashbrown::HashTable;

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) type ShardInner<K, V> = HashTable<(K, V)>;
pub(crate) type ShardReader<'a, K, V> = RwLockReadGuard<'a, ShardInner<K, V>>;
pub(crate) type ShardWriter<'a, K, V> = RwLockWriteGuard<'a, ShardInner<K, V>>;

/// A shard in a [`crate::ShardMap`]. Each shard contains a [`hashbrown::HashTable`] of key-value pairs.
pub(crate) struct Shard<K, V> {
    data: RwLock<ShardInner<K, V>>,
}

pub struct Read<'a, K, V> {
    shard: &'a Shard<K, V>,
}

impl<'a, K, V> Future for Read<'a, K, V> {
    type Output = ShardReader<'a, K, V>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.shard.data.try_read() {
            Ok(guard) => Poll::Ready(guard),
            _ => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

pub struct Write<'a, K, V> {
    shard: &'a Shard<K, V>,
}

impl<'a, K, V> Future for Write<'a, K, V> {
    type Output = ShardWriter<'a, K, V>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.shard.data.try_write() {
            Ok(guard) => Poll::Ready(guard),
            _ => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

impl<K, V> Shard<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            data: RwLock::new(ShardInner::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: RwLock::new(ShardInner::with_capacity(capacity)),
        }
    }

    pub async fn write<'a>(&'a self) -> ShardWriter<'a, K, V> {
        Write { shard: self }.await
    }

    pub async fn read<'a>(&'a self) -> ShardReader<'a, K, V> {
        Read { shard: self }.await
    }
}

impl<K, V> std::ops::Deref for Shard<K, V> {
    type Target = RwLock<ShardInner<K, V>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<K, V> std::ops::DerefMut for Shard<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
