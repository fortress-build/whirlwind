use hashbrown::HashTable;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

pub struct MapRef<'a, 'b, K, V> {
    reader: RwLockReadGuard<'a, HashTable<(K, V)>>,
    key: &'b K,
    hash: u64,
}

impl<K, V> std::ops::Deref for MapRef<'_, '_, K, V>
where
    K: Eq + std::hash::Hash,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.reader
            .find(self.hash, |(k, _)| k == self.key)
            .map(|(_, v)| v)
            .expect("Key not found in mapref but reader was already acquired")
    }
}

impl<'a, 'b, K, V> MapRef<'a, 'b, K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn new(reader: RwLockReadGuard<'a, HashTable<(K, V)>>, key: &'b K, hash: u64) -> Self {
        Self { reader, key, hash }
    }

    pub fn key(&self) -> &K {
        self.key
    }

    pub fn value(&self) -> Option<&V> {
        self.reader
            .find(self.hash, |(k, _)| k == self.key)
            .map(|(_, v)| v)
    }
}

pub struct MapRefMut<'a, 'b, K, V> {
    writer: RwLockWriteGuard<'a, HashTable<(K, V)>>,
    key: &'b K,
    hash: u64,
}

impl<K, V> std::ops::Deref for MapRefMut<'_, '_, K, V>
where
    K: Eq + std::hash::Hash,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.writer
            .find(self.hash, |(k, _)| k == self.key)
            .map(|(_, v)| v)
            .expect("Key not found in mapref but writer was already acquired")
    }
}

impl<K, V> std::ops::DerefMut for MapRefMut<'_, '_, K, V>
where
    K: Eq + std::hash::Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.writer
            .find_mut(self.hash, |(k, _)| k == self.key)
            .map(|(_, v)| v)
            .expect("Key not found in mapref but writer was already acquired")
    }
}

impl<'a, 'b, K, V> MapRefMut<'a, 'b, K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn new(writer: RwLockWriteGuard<'a, HashTable<(K, V)>>, key: &'b K, hash: u64) -> Self {
        Self { writer, key, hash }
    }

    pub fn key(&self) -> &K {
        self.key
    }

    pub fn value(&self) -> Option<&V> {
        self.writer
            .find(self.hash, |(k, _)| k == self.key)
            .map(|(_, v)| v)
    }

    pub fn value_mut(&mut self) -> Option<&mut V> {
        self.writer
            .find_mut(self.hash, |(k, _)| k == self.key)
            .map(|(_, v)| v)
    }
}
