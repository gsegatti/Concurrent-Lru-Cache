# Concurrent LRU Cache

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

The Concurrent LRU Cache is a simple and efficient thread-safe implementation designed on top of `linked_hash_map::LinkedHashMap` and `concurrent_queue::ConcurrentQueue`. It offers efficient caching capabilities while maintaining the Least Recently Used (LRU) property.

## Features

- **Concurrent and Thread-Safe:** This cache allows multiple threads to access it simultaneously without the risk of data corruption or race conditions.

- **LRU Replacement Policy:** The cache efficiently maintains the LRU order of items, ensuring that the most recently used items are retained while evicting the least recently used ones.

- **Lightweight and Simple:** Built with simplicity in mind, this cache provides an easy-to-use interface for managing your data with minimal overhead.

## Installation

To use the Concurrent LRU Cache in your Rust project, add the following line to your `Cargo.toml` file:

```toml
[dependencies]
concurrent-lru-cache = "0.1.0"

## Usage

### Create a Cache

To use the Concurrent LRU Cache, first, create an instance of it with a defined maximum capacity.

```rust
use concurrent_lru_cache::ConcurrentLruCache;

// Create a cache with a maximum capacity of 100 items
let mut cache: ConcurrentLruCache<i32, String> = ConcurrentLruCache::new(100);

### Insert and Retrieve Items

You can insert key-value pairs into the cache and later retrieve the values based on their keys.

```rust
use concurrent_lru_cache::ConcurrentLruCache;

// Create a cache with a maximum capacity of 100 items
let mut cache: ConcurrentLruCache<i32, String> = ConcurrentLruCache::new(100);

// Insert key-value pairs into the cache
cache.put_refresh(1, "Value for key 1".to_string());
cache.put_refresh(2, "Value for key 2".to_string());

// Retrieve values from the cache
if let Some(value) = cache.get(&1) {
    println!("Value for key 1: {}", value);
}

### Refreshing the Cache

To stop the operation queue from becoming too large, you can manually trigger the refreshing of the cache elements.

```rust
use concurrent_lru_cache::ConcurrentLruCache;

// Create a cache with a maximum capacity of 100 items
let mut cache: ConcurrentLruCache<i32, String> = ConcurrentLruCache::new(100);

// Insert key-value pairs into the cache, while refreshing it as well.
cache.put_refresh(1, "Value for key 1".to_string());
cache.put_refresh(2, "Value for key 2".to_string());

// Access an item which will eventually update its ordering within the cache.
cache.get(&1);

// Manually refresh the cache to ensure correct LRU ordering
cache.refresh();

