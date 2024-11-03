use std::hash::BuildHasher;

enum IterState<'a, K, V> {
    Start,
    Shard(
        usize,
        hashbrown::raw::RawIter<(K, V)>,
        crate::shard::ShardReader<'a, K, V>,
    ),
    Finished,
}

pub struct Iter<'a, K, V, S> {
    map: crate::ShardMap<K, V, S>,
    state: IterState<'a, K, V>,
}

impl<'a, K: 'static, V: 'static, S: BuildHasher> Iter<'a, K, V, S>
where
    K: Eq + std::hash::Hash + 'static,
    V: 'static,
{
    pub fn new(map: crate::ShardMap<K, V, S>) -> Self {
        Self {
            map,
            state: IterState::Start,
        }
    }
}

impl<'a, K: Clone + 'static, V: 'static, S: BuildHasher> Iterator for Iter<'a, K, V, S>
where
    K: Eq + std::hash::Hash + 'static,
    V: 'static,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            IterState::Start => {
                let map = self.map.shard_by_idx(0);

                let reader: std::sync::RwLockReadGuard<'static, hashbrown::raw::RawTable<(K, V)>> =
                    unsafe { std::mem::transmute(map.read_sync()) };
                self.state = IterState::Shard(0, unsafe { reader.iter() }, reader);

                self.next()
            }
            IterState::Shard(idx, iter, _shard) => {
                match iter
                    .next()
                    .map(|bucket| unsafe { bucket.as_ref() })
                    .map(|bucket| (&bucket.0, &bucket.1))
                {
                    Some(value) => Some(value),
                    None => {
                        if *idx >= self.map.num_shards() - 1 {
                            self.state = IterState::Finished;
                            return None;
                        }

                        let map = self.map.shard_by_idx(*idx + 1);

                        let reader: std::sync::RwLockReadGuard<
                            'static,
                            hashbrown::raw::RawTable<(K, V)>,
                        > = unsafe { std::mem::transmute(map.read_sync()) };

                        self.state = IterState::Shard(*idx + 1, unsafe { reader.iter() }, reader);

                        self.next()
                    }
                }
            }
            IterState::Finished => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_iter() {
        let map = crate::ShardMap::new();
        map.insert("foo", "bar").await;
        map.insert("baz", "qux").await;
        let mut iter = map.iter().collect::<Vec<_>>();
        iter.sort_by(|a, b| a.0.cmp(b.0));

        assert_eq!(iter[0], (&"baz", &"qux"));
        assert_eq!(iter[1], (&"foo", &"bar"));

        assert_eq!(iter.len(), 2);
    }

    #[tokio::test]
    async fn test_iter_empty() {
        let map = crate::ShardMap::<u32, u32>::new();

        let mut iter = map.iter();
        assert_eq!(iter.next(), None);
    }
}
