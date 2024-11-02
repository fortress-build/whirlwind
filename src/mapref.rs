//! This module contains the `MapRef` and `MapRefMut` types, which are used to hold references to
//! key-value pairs in a [`ShardMap`]. These types are used to ensure that the shard associated with
//! the key is locked for the duration of the reference.
//!
//! # Example
//! ```
//! use whirlwind::ShardMap;
//! use tokio::runtime::Runtime;
//! use std::sync::Arc;
//!
//! let rt = Runtime::new().unwrap();
//! let map = Arc::new(ShardMap::new());
//! rt.block_on(async {
//!     map.insert("foo", "bar").await;
//!     let r = map.get(&"foo").await.unwrap();
//!     assert_eq!(r.key(), &"foo");
//!     assert_eq!(r.value(), &"bar");
//!     drop(r); // release the lock so we can mutate the value.
//!     let mut mr = map.get_mut(&"foo").await.unwrap();
//!     *mr.value_mut() = "baz";
//!     assert_eq!(mr.value(), &"baz");
//! });

use hashbrown::HashTable;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// A reference to a key-value pair in a [`crate::ShardMap`]. Holds a shared (read-only) lock on the shard
/// associated with the key.
pub struct MapRef<'a, 'b, K, V> {
    key: &'b K,
    value: &'b V,
    #[allow(unused)]
    reader: RwLockReadGuard<'a, HashTable<(K, V)>>,
}

impl<K, V> std::ops::Deref for MapRef<'_, '_, K, V>
where
    K: Eq + std::hash::Hash,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, 'b, K, V> MapRef<'a, 'b, K, V>
where
    K: Eq + std::hash::Hash,
{
    pub(crate) fn new(
        reader: RwLockReadGuard<'a, HashTable<(K, V)>>,
        key: &'b K,
        value: &'b V,
    ) -> Self {
        Self { reader, key, value }
    }

    pub fn key(&self) -> &K {
        self.key
    }

    pub fn value(&self) -> &V {
        self.value
    }

    pub fn pair(&self) -> (&K, &V) {
        (self.key, self.value)
    }
}

/// A mutable reference to a key-value pair in a [`crate::ShardMap`]. Holds an exclusive lock on
/// the shard associated with the key.
pub struct MapRefMut<'a, 'b, K, V> {
    key: &'b K,
    value: &'b mut V,
    #[allow(unused)]
    writer: RwLockWriteGuard<'a, HashTable<(K, V)>>,
}

impl<K, V> std::ops::Deref for MapRefMut<'_, '_, K, V>
where
    K: Eq + std::hash::Hash,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<K, V> std::ops::DerefMut for MapRefMut<'_, '_, K, V>
where
    K: Eq + std::hash::Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<'a, 'b, K, V> MapRefMut<'a, 'b, K, V>
where
    K: Eq + std::hash::Hash,
{
    pub(crate) fn new(
        writer: RwLockWriteGuard<'a, HashTable<(K, V)>>,
        key: &'b K,
        value: &'b mut V,
    ) -> Self {
        Self { writer, key, value }
    }

    /// Returns a reference to the key.
    pub fn key(&self) -> &K {
        self.key
    }

    /// returns a reference to the value.
    pub fn value(&self) -> &V {
        self.value
    }

    /// Returns a mutable reference to the value.
    pub fn value_mut(&mut self) -> &mut V {
        self.value
    }

    /// Returns a reference to the key-value pair.
    pub fn pair(&self) -> (&K, &V) {
        (self.key, self.value)
    }

    /// Returns a reference to the key-value pair, with a mutable reference to the value.
    pub fn pair_mut(&mut self) -> (&K, &mut V) {
        (self.key, self.value)
    }
}
