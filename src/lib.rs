use concurrent_queue::ConcurrentQueue;
use linked_hash_map::LinkedHashMap;
use std::{borrow::Borrow, collections::hash_map::RandomState, hash::Hash};

pub struct Cache<K: Eq + Hash + Send + Sync, V: Send + Sync> {
    map: LinkedHashMap<K, V, RandomState>,
    queue: ConcurrentQueue<K>,
    max_size: usize,
}

impl<K: Eq + Hash + Send + Clone + Sync, V: Send + Sync> Cache<K, V> {
    pub fn new(max_size: usize) -> Self {
        Cache {
            map: LinkedHashMap::with_capacity(max_size),
            queue: ConcurrentQueue::unbounded(),
            max_size,
        }
    }
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q> + From<Q>,
        Q: Hash + Eq + Clone,
    {
        self.add_op(k);
        if let Some(v) = self.map.get(k) {
            return Some(v);
        }
        None
    }

    fn put(&mut self, k: K, v: V) -> Option<V> {
        self.add_op(&k);
        let old_val = self.map.insert(k, v);
        if self.len() > self.capacity() {
            self.remove_lru();
        }
        old_val
    }

    pub fn put_refresh(&mut self, k: K, v: V) -> Option<V> {
        self.refresh();
        self.put(k, v)
    }

    pub fn refresh(&mut self) {
        while let Some(key) = self.pop_op() {
            self.map.get_refresh(&key);
        }
    }

    pub fn contains(&self, k: &K) -> bool {
        self.map.contains_key(k)
    }

    pub fn remove_lru(&mut self) -> Option<(K, V)> {
        self.map.pop_front()
    }

    pub fn add_op<Q>(&self, op: &Q)
    where
        K: Borrow<Q> + From<Q>,
        Q: Hash + Eq + Clone,
    {
        self.queue.push(op.clone().into()).ok();
    }

    pub fn pop_op(&self) -> Option<K> {
        if self.queue.is_empty() {
            return None;
        }
        self.queue.pop().ok()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn capacity(&self) -> usize {
        self.max_size
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.refresh();
    }

    pub fn iter(&self) -> linked_hash_map::Iter<K, V> {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> linked_hash_map::IterMut<K, V> {
        self.map.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    #[test]
    fn test_put() {
        let mut cache = super::Cache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_put_and_get() {
        let mut cache = super::Cache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        assert_eq!(cache.get(&1), Some(&1));
        assert_eq!(cache.get(&2), Some(&2));
    }

    #[test]
    fn test_put_update() {
        let mut cache = super::Cache::new(2);
        cache.put(1, 1);
        cache.put(1, 3);
        assert_eq!(cache.get(&1), Some(&3));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_put_over_capacity() {
        let mut cache = super::Cache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        cache.put(3, 3);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&2));
        assert_eq!(cache.get(&3), Some(&3));
    }

    #[test]
    fn test_put_does_not_evict_oldest() {
        let mut cache = super::Cache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        // This should make the entry 2 the oldest.
        cache.get(&1);
        cache.put(3, 3);
        assert_eq!(cache.len(), 2);
        // Assert the entry 1 was evicted instead.
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&2));
        assert_eq!(cache.get(&3), Some(&3));
    }

    #[test]
    fn test_put_refresh_evicts_oldest() {
        let mut cache = super::Cache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        cache.get(&1);
        cache.put_refresh(3, 3);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&1), Some(&1));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&3));
    }

    #[test]
    fn test_contains() {
        let mut cache = super::Cache::new(2);
        cache.put(1, 10);
        assert_eq!(cache.contains(&1), true);
        assert_eq!(cache.contains(&2), false);
    }

    #[test]
    fn test_clear() {
        let mut cache = super::Cache::new(2);
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
