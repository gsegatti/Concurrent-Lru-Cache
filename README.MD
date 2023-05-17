# Concurrent-Lru

A concurrent LRU cache with strong consistency.

The cache leverages `linked_hash_map::LinkedHashMap` does not change under read operations. The order of the read items is stored within a lock-free structure provided by `concurrent_queue::ConcurrentQueue`.

Whenever there's a write operation, the cache updates the ordering of its items.

Therefore, the cache can be wrapped by an `RwLock` to provide concurrent read operations.

## Development and Contributing
> **Warning**
> This is still under development, so breaking changes may occur while doing so.

I intend to further develop and review this crate, but if you want to contribute, feel free to open an issue or a pull request.

## License

Licensed under:
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
