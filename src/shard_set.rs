//! # ShardSet
//!
//! A concurrent set based on a [`ShardMap`] with values of `()`.
//!
//! # Example
//!
//! ```
//! use std::sync::Arc;
//! use whirlwind::ShardSet;
//!
//! let rt = tokio::runtime::Runtime::new().unwrap();
//! let set = Arc::new(ShardSet::new());
//!
//! rt.block_on(async {
//!     for i in 0..10 {
//!         let k = i;
//!         if i % 2 == 0 {
//!             set.insert(k).await;
//!         } else {
//!             set.remove(&(k-1)).await;
//!         }
//!     }
//! });
//! ```
//!
use std::hash::{BuildHasher, Hash, RandomState};

use crate::shard_map::ShardMap;

/// A concurrent set based on a [`ShardMap`] with values of `()`.
pub struct ShardSet<T, S = RandomState> {
    inner: ShardMap<T, (), S>,
}

impl<T: Eq + Hash + 'static> ShardSet<T, RandomState> {
    pub fn new() -> Self {
        Self {
            inner: ShardMap::new(),
        }
    }

    pub fn new_with_shards(shards: usize) -> Self {
        Self {
            inner: ShardMap::new_with_shards(shards),
        }
    }
}

impl<T, S> ShardSet<T, S>
where
    T: Eq + std::hash::Hash + 'static,
    S: BuildHasher,
{
    pub fn new_with_hasher(hasher: S) -> Self {
        Self {
            inner: ShardMap::new_with_hasher(hasher),
        }
    }

    pub fn new_with_shards_and_hasher(shards: usize, hasher: S) -> Self {
        Self {
            inner: ShardMap::new_with_shards_and_hasher(shards, hasher),
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
