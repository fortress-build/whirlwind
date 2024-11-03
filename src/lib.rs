//! # Whirlwind
//!
//! A collection of data structures that allow for concurrent access to shared data.
//!
//! Currently, this crate provides the following data structures:
//!
//! - [`ShardMap`]: A concurrent hashmap using a sharding strategy.
//! - [`ShardSet`]: A concurrent set based on a [`ShardMap`] with values of `()`.
//!
//! ## ShardMap
//!
//! A concurrent hashmap using a sharding strategy.
//!
//! ### Example
//!
//! ```rust
//! use tokio::runtime::Runtime;
//! use std::sync::Arc;
//! use whirlwind::ShardMap;
//!
//! let rt = Runtime::new().unwrap();
//! let map = Arc::new(ShardMap::new());
//!
//! rt.block_on(async {
//!    map.insert("foo", "bar").await;
//!    assert_eq!(map.len().await, 1);
//!    assert_eq!(map.contains_key(&"foo").await, true);
//! });
//! ```
//!
//! ## ShardSet
//!
//! A concurrent set based on a [`ShardMap`] with values of `()`.
//!
//! ### Example
//!
//! ```rust
//! use tokio::runtime::Runtime;
//! use std::sync::Arc;
//! use whirlwind::ShardSet;
//!
//! let rt = Runtime::new().unwrap();
//! let set = Arc::new(ShardSet::new());
//! rt.block_on(async {
//!     set.insert("foo").await;
//!     assert_eq!(set.contains(&"foo").await, true);
//!     set.remove(&"foo").await;
//!     assert_eq!(set.contains(&"foo").await, false);
//!     assert_eq!(set.len().await, 0);
//! });
//! ```
//!
//!
//! See the documentation for each data structure for more information.

pub mod iter;
pub mod mapref;
mod shard;
mod shard_map;
mod shard_set;

pub use shard_map::ShardMap;
pub use shard_set::ShardSet;
