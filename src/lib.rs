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
