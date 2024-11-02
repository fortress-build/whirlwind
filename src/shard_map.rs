use std::{
    hash::Hasher,
    sync::{atomic::AtomicUsize, Arc},
};

use crate::{
    mapref::{MapRef, MapRefMut},
    shard::Shard,
};

struct Inner<K, V> {
    shards: Vec<Shard<K, V>>,
    length: AtomicUsize,
    key: std::marker::PhantomData<K>,
}

impl<K, V> std::ops::Deref for Inner<K, V> {
    type Target = Vec<Shard<K, V>>;

    fn deref(&self) -> &Self::Target {
        &self.shards
    }
}

impl<K, V> std::ops::DerefMut for Inner<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shards
    }
}

/// A concurrent hashmap using a sharding strategy.
pub struct ShardMap<K, V> {
    inner: Arc<Inner<K, V>>,
}

impl<K, V> Clone for ShardMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<K, V> ShardMap<K, V>
where
    K: Eq + std::hash::Hash + 'static,
    V: 'static,
{
    pub fn new() -> Self {
        let n_shards = num_cpus::get().max(4);
        let shards = std::iter::repeat_n((), n_shards)
            .map(|_| Shard::new())
            .collect();

        Self {
            inner: Arc::new(Inner {
                shards,
                length: AtomicUsize::new(0),
                key: std::marker::PhantomData,
            }),
        }
    }

    pub fn new_with_shards(shards: usize) -> Self {
        let shards = std::iter::repeat_n((), shards)
            .map(|_| Shard::new())
            .collect();
        Self {
            inner: Arc::new(Inner {
                shards,
                length: AtomicUsize::new(0),
                key: std::marker::PhantomData,
            }),
        }
    }

    fn shard(&self, key: &K) -> (&Shard<K, V>, u64) {
        // TODO: Allow custom hashing, maybe don't create a new hasher every time
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let shard_idx = (hash % Vec::len(&self.inner) as u64) as usize;
        (&self.inner[shard_idx], hash)
    }

    pub async fn insert(&self, key: K, value: V) -> Option<V> {
        let (shard, hash) = self.shard(&key);
        let mut writer = shard.write().await;
        let old = writer.entry(
            hash,
            |(k, _)| k == &key,
            |(k, _)| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                k.hash(&mut hasher);
                hasher.finish()
            },
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

    pub async fn get<'a, 'b: 'a>(&'a self, key: &'b K) -> Option<MapRef<'_, '_, K, V>> {
        let (shard, hash) = self.shard(key);

        let reader = shard.read().await;

        reader
            .find(hash, |(k, _)| k == key)
            .map(|_| ())
            .map(|_| MapRef::new(reader, key, hash))
    }

    pub async fn get_mut<'a, 'b: 'a>(&'a self, key: &'b K) -> Option<MapRefMut<'_, '_, K, V>> {
        let (shard, hash) = self.shard(key);
        let writer = shard.write().await;
        writer
            .find(hash, |(k, _)| k == key)
            .map(|_| ())
            .map(|_| MapRefMut::new(writer, key, hash))
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
