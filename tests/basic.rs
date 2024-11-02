use whirlwind::*;

#[tokio::test]
async fn test_shardmap() {
    let map = ShardMap::new();
    map.insert("foo", "bar").await;
    assert_eq!(map.len(), 1);
    assert_eq!(map.contains_key(&"foo").await, true);
    assert_eq!(map.contains_key(&"bar").await, false);
    assert_eq!(map.get(&"foo").await.unwrap().value(), &"bar");
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
    assert_eq!(map2.get(&"foo").await.unwrap().value(), &"bar");
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
    assert_eq!(map.get(&"foo").await.unwrap().value(), &"bar");
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
