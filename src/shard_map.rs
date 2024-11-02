//! A concurrent hashmap using a sharding strategy.
//!
//! # Examples
//! ```
//! use tokio::runtime::Runtime;
//! use std::sync::Arc;
//! use whirlwind::ShardMap;
//!
//! let rt = Runtime::new().unwrap();
//! let map = Arc::new(ShardMap::new());
//! rt.block_on(async {
//!    map.insert("foo", "bar").await;
//!    assert_eq!(map.len(), 1);
//!    assert_eq!(map.contains_key(&"foo").await, true);
//!    assert_eq!(map.contains_key(&"bar").await, false);
//!
//!    assert_eq!(map.get(&"foo").await.unwrap().value(), &"bar");
//!    assert_eq!(map.remove(&"foo").await, Some("bar"));
//! });
//!
use std::{
    hash::{BuildHasher, RandomState},
    sync::{atomic::AtomicUsize, Arc, OnceLock},
};

use crate::{
    mapref::{MapRef, MapRefMut},
    shard::Shard,
};

struct Inner<K, V, S = RandomState> {
    shards: Vec<Shard<K, V>>,
    length: AtomicUsize,
    hasher: S,
}

impl<K, V, S> std::ops::Deref for Inner<K, V, S> {
    type Target = Vec<Shard<K, V>>;

    fn deref(&self) -> &Self::Target {
        &self.shards
    }
}

impl<K, V, S> std::ops::DerefMut for Inner<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shards
    }
}

/// A concurrent hashmap using a sharding strategy.
pub struct ShardMap<K, V, S = std::hash::RandomState> {
    inner: Arc<Inner<K, V, S>>,
}

impl<K, V, H: BuildHasher> Clone for ShardMap<K, V, H> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

fn shard_count() -> usize {
    // Same as DashMap
    static SHARD_COUNT: OnceLock<usize> = OnceLock::new();
    *SHARD_COUNT.get_or_init(|| {
        (std::thread::available_parallelism().map_or(1, usize::from) * 4).next_power_of_two()
    })
}

impl<K, V> ShardMap<K, V, RandomState>
where
    K: Eq + std::hash::Hash + 'static,
    V: 'static,
{
    pub fn new() -> Self {
        Self::new_with_shards(shard_count())
    }

    pub fn new_with_shards(shards: usize) -> Self {
        Self::new_with_shards_and_hasher(shards, RandomState::new())
    }
}

impl<K, V, S: BuildHasher> ShardMap<K, V, S>
where
    K: Eq + std::hash::Hash + 'static,
    V: 'static,
{
    pub fn new_with_hasher(hasher: S) -> Self {
        Self::new_with_shards_and_hasher(shard_count(), hasher)
    }

    pub fn new_with_shards_and_hasher(shards: usize, hasher: S) -> Self {
        let shards = std::iter::repeat(())
            .take(shards)
            .map(|_| Shard::new())
            .collect();

        Self {
            inner: Arc::new(Inner {
                shards,
                length: AtomicUsize::new(0),
                hasher,
            }),
        }
    }

    fn shard(&self, key: &K) -> (&Shard<K, V>, u64) {
        let hash = self.inner.hasher.hash_one(key);
        let shard_idx = (hash % Vec::len(&self.inner) as u64) as usize;
        (&self.inner[shard_idx], hash)
    }

    pub async fn insert(&self, key: K, value: V) -> Option<V> {
        let (shard, hash) = self.shard(&key);
        let mut writer = shard.write().await;
        let old = writer.entry(
            hash,
            |(k, _)| k == &key,
            |(k, _)| self.inner.hasher.hash_one(k),
        );
        match old {
            hashbrown::hash_table::Entry::Occupied(o) => {
                let (old, vacant) = o.remove();
                vacant.insert((key, value));
                Some(old.1)
            }
            hashbrown::hash_table::Entry::Vacant(v) => {
                v.insert((key, value));

                self.inner
                    .length
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                None
            }
        }
    }

    pub async fn get<'a, 'b: 'a>(&'a self, key: &'b K) -> Option<MapRef<'a, 'b, K, V>> {
        let (shard, hash) = self.shard(key);

        let reader = shard.read().await;

        reader
            .find(hash, |(k, _)| k == key)
            .map(|(k, v)| (k as *const K, v as *const V))
            .map(|(k, v)| unsafe {
                // SAFETY: The key and value are guaranteed to be valid for the lifetime of the reader.
                MapRef::new(reader, &*k, &*v)
            })
    }

    pub async fn get_mut<'a, 'b: 'a>(&'a self, key: &'b K) -> Option<MapRefMut<'a, 'b, K, V>> {
        let (shard, hash) = self.shard(key);
        let mut writer = shard.write().await;
        writer
            .find_mut(hash, |(k, _)| k == key)
            .map(|(k, v)| (k as *const K, v as *mut V))
            .map(|(k, v)| unsafe {
                // SAFETY: The key and value are guaranteed to be valid for the lifetime of the writer.
                MapRefMut::new(writer, &*k, &mut *v)
            })
    }

    pub async fn contains_key(&self, key: &K) -> bool {
        let (shard, hash) = self.shard(key);

        let reader = shard.read().await;
        reader.find(hash, |(k, _)| k == key).is_some()
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let (shard, hash) = self.shard(key);
        let mut shard = shard.write().await;
        match shard.find_entry(hash, |(k, _)| k == key) {
            Ok(v) => {
                let ((_, v), _) = v.remove();

                self.inner
                    .length
                    .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

                Some(v)
            }
            Err(_) => None,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.length.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub async fn clear(&self) {
        for shard in self.inner.iter() {
            shard.write().await.clear();
        }
    }
}
