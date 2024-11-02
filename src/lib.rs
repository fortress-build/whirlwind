//! A collection of data structures that allow for concurrent access to shared data.
//!
//! Currently, this crate provides the following data structures:
//!
//! - [`ShardMap`]: A concurrent hashmap using a sharding strategy.
//! - [`ShardSet`]: A concurrent set based on a [`ShardMap`] with values of `()`.
//!
//! See the documentation for each data structure for more information.

pub mod mapref;
mod shard;
pub mod shard_map;
pub mod shard_set;

pub use shard_map::ShardMap;
pub use shard_set::ShardSet;
