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

use crate::shard::{ShardReader, ShardWriter};

/// A reference to a key-value pair in a [`crate::ShardMap`]. Holds a shared (read-only) lock on the shard
/// associated with the key.
pub struct MapRef<'a, K, V> {
    key: &'a K,
    value: &'a V,
    #[allow(unused)]
    reader: ShardReader<'a, K, V>,
}

impl<K, V> std::ops::Deref for MapRef<'_, K, V>
where
    K: Eq + std::hash::Hash,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, K, V> MapRef<'a, K, V>
where
    K: Eq + std::hash::Hash,
{
    pub(crate) fn new(reader: ShardReader<'a, K, V>, key: &'a K, value: &'a V) -> Self {
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
pub struct MapRefMut<'a, K, V> {
    key: &'a K,
    value: &'a mut V,
    #[allow(unused)]
    writer: ShardWriter<'a, K, V>,
}

impl<'a, K, V> std::ops::Deref for MapRefMut<'a, K, V>
where
    K: Eq + std::hash::Hash,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, K, V> std::ops::DerefMut for MapRefMut<'a, K, V>
where
    K: Eq + std::hash::Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<'a, K, V> MapRefMut<'a, K, V>
where
    K: Eq + std::hash::Hash,
{
    pub(crate) fn new(writer: ShardWriter<'a, K, V>, key: &'a K, value: &'a mut V) -> Self {
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
