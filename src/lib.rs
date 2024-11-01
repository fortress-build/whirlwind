use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{atomic::AtomicUsize, Arc},
};

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

struct Shard<K, V> {
    data: RwLock<HashMap<K, V>>,
}

impl<K, V> Shard<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl<K, V> std::ops::Deref for Shard<K, V> {
    type Target = RwLock<HashMap<K, V>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<K, V> std::ops::DerefMut for Shard<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

struct Inner<K, V> {
    shards: Vec<Shard<K, V>>,
    length: AtomicUsize,
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

pub struct ShardMap<K, V> {
    inner: Arc<Inner<K, V>>,
}

impl<K, V> Clone for ShardMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> ShardMap<K, V>
where
    K: Eq + Hash + 'static,
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
            }),
        }
    }

    fn shard(&self, key: &K) -> &Shard<K, V> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let shard_idx = (hasher.finish() % self.inner.len() as u64) as usize;
        &self.inner[shard_idx]
    }

    pub async fn insert(&self, key: K, value: V) {
        let shard = self.shard(&key);
        let mut shard = shard.write().await;
        match shard.insert(key, value) {
            Some(_) => {}
            None => {
                self.inner
                    .length
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    pub async fn get<'a, 'b: 'a>(&'a self, key: &'b K) -> Option<MapRef<'_, '_, K, V>> {
        let reader = self.shard(key).read().await;
        reader
            .contains_key(key)
            .then(move || MapRef { reader, key })
    }

    pub async fn get_mut<'a, 'b: 'a>(&'a self, key: &'b K) -> Option<MapRefMut<'_, '_, K, V>> {
        let writer = self.shard(key).write().await;
        writer
            .contains_key(key)
            .then(move || MapRefMut { writer, key })
    }

    pub async fn contains_key(&self, key: &K) -> bool {
        self.shard(key).read().await.contains_key(key)
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let shard = self.shard(key);
        let mut shard = shard.write().await;
        shard.remove(key).and_then(|v| {
            self.inner
                .length
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            Some(v)
        })
    }

    pub fn len(&self) -> usize {
        // let mut sum = 0;
        //
        // let mut set = tokio::task::JoinSet::new();
        //
        // let count = self.inner.len();
        // for shard in 0..count {
        //     let this = self.clone();
        //     set.spawn_local(async move { this.inner[shard].read().await.len() });
        // }
        //
        // while let Some(count) = set.join_next().await.transpose().expect("join error") {
        //     sum += count;
        // }
        //
        // sum
        self.inner.length.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct MapRef<'a, 'b, K, V> {
    reader: RwLockReadGuard<'a, HashMap<K, V>>,
    key: &'b K,
}

impl<K, V> std::ops::Deref for MapRef<'_, '_, K, V>
where
    K: Eq + Hash,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.reader
            .get(self.key)
            .expect("Key not found in mapref but reader was already acquired")
    }
}

impl<'a, 'b, K, V> MapRef<'a, 'b, K, V>
where
    K: Eq + Hash,
{
    pub fn key(&self) -> &K {
        self.key
    }

    pub fn value(&self) -> Option<&V> {
        self.reader.get(self.key)
    }
}

pub struct MapRefMut<'a, 'b, K, V> {
    writer: RwLockWriteGuard<'a, HashMap<K, V>>,
    key: &'b K,
}

impl<K, V> std::ops::Deref for MapRefMut<'_, '_, K, V>
where
    K: Eq + Hash,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.writer
            .get(self.key)
            .expect("Key not found in mapref but writer was already acquired")
    }
}

impl<K, V> std::ops::DerefMut for MapRefMut<'_, '_, K, V>
where
    K: Eq + Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.writer
            .get_mut(self.key)
            .expect("Key not found in mapref but writer was already acquired")
    }
}

impl<'a, 'b, K, V> MapRefMut<'a, 'b, K, V>
where
    K: Eq + Hash,
{
    pub fn key(&self) -> &K {
        self.key
    }

    pub fn value(&self) -> Option<&V> {
        self.writer.get(self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shardmap() {
        let map = ShardMap::new();
        map.insert("foo", "bar").await;
        assert_eq!(map.len(), 1);
        assert_eq!(map.contains_key(&"foo").await, true);
        assert_eq!(map.contains_key(&"bar").await, false);
        assert_eq!(map.get(&"foo").await.unwrap().value(), Some(&"bar"));
        assert!(map.get(&"bar").await.is_none());
        assert_eq!(map.remove(&"foo").await, Some("bar"));
        assert_eq!(map.len(), 0);
        assert_eq!(map.contains_key(&"foo").await, false);
    }

    #[tokio::test]
    async fn test_shardmap_clone() {
        let map = ShardMap::new();
        map.insert("foo", "bar").await;
        let map2 = map.clone();
        assert_eq!(map2.len(), 1);
        assert_eq!(map2.contains_key(&"foo").await, true);
        assert_eq!(map2.contains_key(&"bar").await, false);
        assert_eq!(map2.get(&"foo").await.unwrap().value(), Some(&"bar"));
        assert!(map2.get(&"bar").await.is_none());
        assert_eq!(map2.remove(&"foo").await, Some("bar"));
        assert_eq!(map2.len(), 0);
        assert_eq!(map2.contains_key(&"foo").await, false);
    }

    #[tokio::test]
    async fn test_shardmap_shards() {
        let map = ShardMap::new_with_shards(4);
        map.insert("foo", "bar").await;
        assert_eq!(map.len(), 1);
        assert_eq!(map.contains_key(&"foo").await, true);
        assert_eq!(map.contains_key(&"bar").await, false);
        assert_eq!(map.get(&"foo").await.unwrap().value(), Some(&"bar"));
        assert!(map.get(&"bar").await.is_none());
        assert_eq!(map.remove(&"foo").await, Some("bar"));
        assert_eq!(map.len(), 0);
        assert_eq!(map.contains_key(&"foo").await, false);
    }

    #[tokio::test]
    async fn test_shardmap_len() {
        let map = ShardMap::new();
        map.insert("foo", "bar").await;
        assert_eq!(map.len(), 1);
        map.insert("foo2", "bar2").await;
        assert_eq!(map.len(), 2);
        map.remove(&"foo").await;
        assert_eq!(map.len(), 1);
        map.remove(&"foo2").await;
        assert_eq!(map.len(), 0);
    }

    #[tokio::test]
    async fn test_shardmap_is_empty() {
        let map = ShardMap::new();
        assert_eq!(map.is_empty().await, true);
        map.insert("foo", "bar").await;
        assert_eq!(map.is_empty().await, false);
        map.remove(&"foo").await;
        assert_eq!(map.is_empty().await, true);
    }
}
