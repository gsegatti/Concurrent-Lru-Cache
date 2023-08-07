# Concurrent LRU Cache

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

The Concurrent LRU Cache is a simple and efficient thread-safe implementation designed on top of `linked_hash_map::LinkedHashMap` and `concurrent_queue::ConcurrentQueue`. It offers efficient caching capabilities while maintaining the Least Recently Used (LRU) property.

## Features

- **Concurrent and Thread-Safe:** This cache allows multiple threads to access it simultaneously without the risk of data corruption or race conditions.

- **LRU Replacement Policy:** The cache maintains the LRU order of items, ensuring that the most recently used items are retained while evicting the least recently used ones.

- **Simple:** Built with simplicity in mind, this cache provides an easy-to-use interface for managing your data with minimal overhead on top of well established crates.

## Installation

To use the Concurrent LRU Cache in your Rust project, add the following line to your `Cargo.toml` file:

```toml
[dependencies]
concurrent-lru-cache = "0.1.0"
