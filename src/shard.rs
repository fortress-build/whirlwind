use hashbrown::HashTable;
use tokio::sync::RwLock;

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
