use concurrent_queue::ConcurrentQueue;
use linked_hash_map::LinkedHashMap;
use std::{collections::hash_map::RandomState, hash::Hash};

/// An LRU cache with concurrent reads.
pub struct ConcurrentLruCache<'a, K: Eq + Hash + Send + Sync, V: Send + Sync> {
    map: LinkedHashMap<K, V, RandomState>,
    queue: ConcurrentQueue<&'a K>,
    max_size: usize,
}

impl<'a, K: Eq + Hash + Send + Sync, V: Send + Sync> ConcurrentLruCache<'a, K, V> {
    /// Create an empty cache with a maximum size.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(2);
    /// ```
    pub fn new(max_size: usize) -> Self {
        ConcurrentLruCache {
            map: LinkedHashMap::with_capacity(max_size),
            queue: ConcurrentQueue::unbounded(),
            max_size,
        }
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(1);
    ///
    /// cache.put_refresh(1, "a");
    /// assert_eq!(cache.get(&1), Some(&"a"));
    /// assert_eq!(cache.get(&2), None);
    /// ```
    pub fn get(&self, k: &K) -> Option<&V> {
        if let Some((key, val)) = self.map.get_key_value(k) {
            self.add_op(key);
            return Some(val);
        }
        None
    }

    /// Returns a reference to the value corresponding to the key while refreshing the LRU ordering of elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(3);
    ///
    /// cache.put_refresh(1, "a");
    /// cache.put_refresh(2, "b");
    /// cache.get(&1);
    /// assert_eq!(cache.op_queue_len(), 1);
    ///
    /// assert_eq!(cache.get_refresh(&2), Some(&"b"));
    /// assert_eq!(cache.op_queue_len(), 0);
    /// ```
    pub fn get_refresh(&mut self, k: &K) -> Option<&V> {
        self.refresh();
        match self.map.get_refresh(k) {
            Some(val) => Some(val),
            None => None,
        }
    }

    /// Puts the key-value pair into the cache.
    ///
    /// If the key already exists, the value is updated and the old value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(1);
    ///
    /// cache.put_refresh(1, "a");
    /// assert_eq!(cache.get(&1), Some(&"a"));
    /// ```
    fn put(&mut self, k: K, v: V) -> Option<V> {
        if self.len() == self.capacity() {
            self.map.pop_front();
        }
        self.map.insert(k, v)
    }

    /// Puts the key-value pair into the cache while refreshing the ordering of elements.
    ///
    /// If the key already exists, the value is updated and the old value is returned.
    ///
    /// # Examples
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(3);
    ///
    /// cache.put_refresh(1, "a");
    /// cache.put_refresh(2, "b");
    /// cache.put_refresh(1, "aa");
    /// cache.put_refresh(3, "c");
    ///
    /// assert_eq!(cache.remove_lru(), Some((2, "b")));
    /// assert_eq!(cache.remove_lru(), Some((1, "aa")));
    /// assert_eq!(cache.remove_lru(), Some((3, "c")));
    /// assert_eq!(cache.remove_lru(), None);
    /// ```
    pub fn put_refresh(&mut self, k: K, v: V) -> Option<V> {
        self.refresh();
        self.put(k, v)
    }

    /// Refreshes the ordering of elements in the cache, leaving the cache with the correct least recently used ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(2);
    ///
    /// cache.put_refresh(1, "a");
    /// cache.put_refresh(2, "b");
    ///
    /// // This should make the entry 2 the least recently used.
    /// cache.get(&1);
    /// cache.refresh();
    /// assert_eq!(cache.remove_lru(), Some((2, "b")));
    /// ```
    pub fn refresh(&mut self) {
        while let Some(key) = self.pop_op() {
            self.map.get_refresh(key);
        }
    }

    /// Returns `true` if the cache contains a value for the specified key, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(1);
    ///
    /// cache.put_refresh(1, "a");
    ///
    /// assert_eq!(cache.contains(&1), true);
    /// assert_eq!(cache.contains(&2), false);
    /// ```
    pub fn contains(&self, k: &K) -> bool {
        self.map.contains_key(k)
    }

    /// Refreshes the cache, removes and returns the least recently used key-value pair from the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(2);
    ///
    /// cache.put_refresh(1, "a");
    /// cache.put_refresh(2, "b");
    /// cache.get(&1);
    ///
    /// assert_eq!(cache.remove_lru(), Some((2, "b")));
    /// assert_eq!(cache.remove_lru(), Some((1, "a")));
    /// assert_eq!(cache.remove_lru(), None);
    /// ```
    pub fn remove_lru(&mut self) -> Option<(K, V)> {
        self.refresh();
        self.map.pop_front()
    }

    /// Adds an operation to the queue.
    ///
    fn add_op(&self, op: &'a K) {
        self.queue.push(op).ok();
    }

    /// Returns an operation from the queue, if any.
    ///
    fn pop_op(&self) -> Option<&'a K> {
        if self.queue.is_empty() {
            return None;
        }
        self.queue.pop().ok()
    }

    /// Returns the number of elements in the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(2);
    ///
    /// cache.put_refresh(1, "a");
    /// assert_eq!(cache.len(), 1);
    ///
    /// cache.put_refresh(2, "b");
    /// assert_eq!(cache.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns the number of operations in the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(2);
    ///
    /// cache.put_refresh(1, "a");
    /// cache.put_refresh(2, "b");
    /// cache.get(&1);
    /// assert_eq!(cache.op_queue_len(), 1);
    ///
    /// cache.get(&2);
    /// assert_eq!(cache.op_queue_len(), 2);
    /// ```
    pub fn op_queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Returns the maximum number of elements the cache can hold.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(2);
    /// assert_eq!(cache.capacity(), 2);
    /// ```
    pub fn capacity(&self) -> usize {
        self.max_size
    }

    /// Returns `true` if the cache is empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(2);
    /// assert_eq!(cache.is_empty(), true);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns `true` if the operation queue is empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(2);
    /// assert_eq!(cache.is_op_queue_empty(), true);
    /// ```
    pub fn is_op_queue_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Clears the cache, removing all key-value pairs and clearing the operation queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(2);
    ///
    /// cache.put_refresh(1, "a");
    /// cache.put_refresh(2, "b");
    /// cache.clear();
    ///
    /// assert_eq!(cache.len(), 0);
    /// assert_eq!(cache.is_empty(), true);
    /// ```
    pub fn clear(&mut self) {
        self.map.clear();
        self.refresh();
    }

    /// Returns an iterator over the cache's key-value pairs without guarantees of least recently used order.
    ///
    /// Accessing the cache through the iterator does not affect the ordering of elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, &str> = ConcurrentLruCache::new(2);
    ///
    /// cache.put_refresh(1, "a");
    /// cache.put_refresh(2, "b");
    ///
    /// let kvs: Vec<_> = cache.iter().collect();
    /// assert_eq!(kvs, vec![(&1, &"a"), (&2, &"b")]);
    /// ```
    pub fn iter(&self) -> linked_hash_map::Iter<K, V> {
        self.map.iter()
    }

    /// Returns an iterator over the cache's key-value pairs without guarantees of least recently used order.
    ///
    /// Accessing the cache through the iterator does not affect the ordering of elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use concurrent_lru_cache::ConcurrentLruCache;
    /// let mut cache: ConcurrentLruCache<i32, i32> = ConcurrentLruCache::new(2);
    ///
    /// cache.put_refresh(1, 11);
    /// cache.put_refresh(2, 22);
    ///
    /// for (k, v) in cache.iter_mut() {
    ///    *v += 1;
    /// }
    ///
    /// assert_eq!(cache.get(&1), Some(&12));
    /// assert_eq!(cache.get(&2), Some(&23));
    /// ```
    pub fn iter_mut(&mut self) -> linked_hash_map::IterMut<K, V> {
        self.map.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    #[test]
    fn test_put() {
        let mut cache = super::ConcurrentLruCache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_put_and_get() {
        let mut cache = super::ConcurrentLruCache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        assert_eq!(cache.get(&1), Some(&1));
        assert_eq!(cache.get(&2), Some(&2));
    }

    #[test]
    fn test_put_update() {
        let mut cache = super::ConcurrentLruCache::new(2);
        cache.put(1, 1);
        cache.put(1, 3);
        assert_eq!(cache.get(&1), Some(&3));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_put_over_capacity() {
        let mut cache = super::ConcurrentLruCache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        cache.put(3, 3);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&2));
        assert_eq!(cache.get(&3), Some(&3));
    }

    #[test]
    fn test_put_refresh_evicts_oldest() {
        let mut cache = super::ConcurrentLruCache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        // This should make the entry 2 the oldest.
        cache.get(&1);
        cache.put_refresh(3, 3);
        assert_eq!(cache.len(), 2);
        // Assert the entry 2 was evicted instead.
        assert_eq!(cache.get(&1), Some(&1));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&3));
    }

    #[test]
    fn test_contains() {
        let mut cache = super::ConcurrentLruCache::new(2);
        cache.put(1, 10);
        assert_eq!(cache.contains(&1), true);
        assert_eq!(cache.contains(&2), false);
    }

    #[test]
    fn test_clear() {
        let mut cache = super::ConcurrentLruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.get(&1);
        cache.clear();
        assert!(cache.is_empty());
        assert!(cache.queue.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.contains(&1), false);
        assert_eq!(cache.contains(&2), false);
    }
}
