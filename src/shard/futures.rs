use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use super::{Shard, ShardReader, ShardWriter};

/// A future that resolves to a read lock on a shard.
pub(crate) struct Read<'a, K, V> {
    shard: &'a Shard<K, V>,
}

impl<'a, K, V> Read<'a, K, V> {
    pub(crate) fn new(shard: &'a Shard<K, V>) -> Self {
        Self { shard }
    }
}

impl<'a, K, V> Future for Read<'a, K, V> {
    type Output = ShardReader<'a, K, V>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.shard.data.try_read() {
            Ok(guard) => Poll::Ready(guard),
            _ => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

pub(crate) struct Write<'a, K, V> {
    shard: &'a Shard<K, V>,
}

impl<'a, K, V> Write<'a, K, V> {
    pub(crate) fn new(shard: &'a Shard<K, V>) -> Self {
        Self { shard }
    }
}

impl<'a, K, V> Future for Write<'a, K, V> {
    type Output = ShardWriter<'a, K, V>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.shard.data.try_write() {
            Ok(guard) => Poll::Ready(guard),
            _ => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
