# Concurrent LRU Cache

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

The Concurrent LRU Cache is a simple and thread-safe implementation designed on top of `linked_hash_map::LinkedHashMap` and `concurrent_queue::ConcurrentQueue`. It offers caching capabilities while being able to maintain the Least Recently Used (LRU) property.

# Future Implementation

The idea here is to have something that can keep an approximated LRU ordering (which can indeed be updated to *actual* LRU order). Sometimes, we don't care about evicting the one least recently used node, but rather one of the least recently used. Specially when all older values have diminishing worth quickly. I think this is the scenario where this cache would be more efficient then a mutex wrapping a non-thread safe lru cache.

That being said, the little benchmark I did conduct here, whilst maintaining LRU order strictly, had shown very similar results to a mutex wrapped LRU cache. Hence why I believe if we can relax the LRU constraints this cache may be able to surpass it performance-wise. That would, however, yield a perhaps hard decision making process of knowing *when* to update our cache LRU ordering.
