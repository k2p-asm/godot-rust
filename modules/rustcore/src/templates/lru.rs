//! `core/templates/lru.h`: the least-recently-used cache.
//!
//! The engine keeps a front-ordered list plus a map. `insert` puts the new pair
//! at the front, drops any previous pair with the same key (calling the
//! before-evict hook on it) and trims from the back while the size exceeds the
//! capacity. `get`/`get_ptr` mark the entry as most recent; `erase` removes it.
//!
//! The Rust version stores `(key, value)` pairs front-to-back in a `VecDeque`,
//! which reproduces the same ordering and eviction without the intrusive list.
//! The default capacity is 64 and `set_capacity` only applies when the current
//! capacity is non-zero (the engine's guard).

use std::collections::VecDeque;

pub struct LruCache<K, V> {
    /// Front is the most recently used entry.
    entries: VecDeque<(K, V)>,
    capacity: usize,
}

impl<K: PartialEq, V> Default for LruCache<K, V> {
    fn default() -> Self {
        Self::new(64)
    }
}

impl<K: PartialEq, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            capacity,
        }
    }

    /// `insert`: replaces the key (evicting the old pair) and trims the tail.
    /// Returns the stored value, or `None` when the capacity is zero (the engine
    /// would return a reference to the pair it just evicted).
    pub fn insert(&mut self, key: K, value: V) -> Option<&V> {
        if let Some(index) = self.position(&key) {
            self.entries.remove(index);
        }
        self.entries.push_front((key, value));
        while self.entries.len() > self.capacity {
            self.entries.pop_back();
        }
        self.entries.front().map(|(_, value)| value)
    }

    fn position(&self, key: &K) -> Option<usize> {
        self.entries.iter().position(|(k, _)| k == key)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn has(&self, key: &K) -> bool {
        self.position(key).is_some()
    }

    /// `erase`: removes the key, reporting whether it was present.
    pub fn erase(&mut self, key: &K) -> bool {
        match self.position(key) {
            Some(index) => {
                self.entries.remove(index);
                true
            }
            None => false,
        }
    }

    /// `get`: marks the entry most recent and returns its value.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let index = self.position(key)?;
        let pair = self.entries.remove(index)?;
        self.entries.push_front(pair);
        self.entries.front().map(|(_, value)| value)
    }

    /// `getptr`: the same as [`Self::get`] (`None` instead of crashing when the
    /// key is missing, which is what every caller in the engine relies on).
    pub fn get_ptr(&mut self, key: &K) -> Option<&V> {
        self.get(key)
    }

    /// A read-only lookup that does not touch the recency order.
    pub fn peek(&self, key: &K) -> Option<&V> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, value)| value)
    }

    pub fn get_capacity(&self) -> usize {
        self.capacity
    }

    pub fn get_size(&self) -> usize {
        self.entries.len()
    }

    /// Ignored when the current capacity is zero, like the engine.
    pub fn set_capacity(&mut self, capacity: usize) {
        if self.capacity > 0 {
            self.capacity = capacity;
            while self.entries.len() > self.capacity {
                self.entries.pop_back();
            }
        }
    }

    /// The keys in recency order (most recent first); handy for tests.
    pub fn keys(&self) -> Vec<&K> {
        self.entries.iter().map(|(k, _)| k).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_lookup() {
        let mut cache: LruCache<i32, &str> = LruCache::new(2);
        assert_eq!(cache.insert(1, "one"), Some(&"one"));
        assert_eq!(cache.insert(2, "two"), Some(&"two"));
        assert_eq!(cache.get_size(), 2);
        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), None);
        assert!(cache.has(&1));
        assert!(!cache.has(&3));
    }

    #[test]
    fn old_entries_evict_from_the_back() {
        let mut cache: LruCache<i32, i32> = LruCache::new(2);
        cache.insert(1, 10);
        cache.insert(2, 20);
        // Reading 1 makes 2 the least recent, so inserting 3 evicts 2.
        assert_eq!(cache.get(&1), Some(&10));
        cache.insert(3, 30);
        assert_eq!(cache.get_size(), 2);
        assert!(cache.has(&1));
        assert!(!cache.has(&2));
        assert!(cache.has(&3));
    }

    #[test]
    fn inserting_an_existing_key_replaces_it() {
        let mut cache: LruCache<i32, i32> = LruCache::new(3);
        cache.insert(1, 10);
        cache.insert(2, 20);
        cache.insert(1, 11);
        assert_eq!(cache.get_size(), 2);
        assert_eq!(cache.get(&1), Some(&11));
        assert_eq!(cache.keys(), vec![&1, &2]);
    }

    #[test]
    fn erase_and_clear() {
        let mut cache: LruCache<i32, i32> = LruCache::new(4);
        cache.insert(1, 10);
        cache.insert(2, 20);
        assert!(cache.erase(&1));
        assert!(!cache.erase(&1));
        assert_eq!(cache.get_size(), 1);
        cache.clear();
        assert_eq!(cache.get_size(), 0);
        assert!(cache.keys().is_empty());
    }

    #[test]
    fn capacity_rules() {
        // The default capacity is 64.
        let cache: LruCache<i32, i32> = LruCache::default();
        assert_eq!(cache.get_capacity(), 64);

        let mut small: LruCache<i32, i32> = LruCache::new(1);
        small.insert(1, 1);
        small.insert(2, 2);
        assert_eq!(small.get_size(), 1);
        assert_eq!(small.get(&2), Some(&2));
        assert!(!small.has(&1));

        // Shrinking evicts the tail (oldest first): 3 and 2 are the most recent.
        let mut cache: LruCache<i32, i32> = LruCache::new(4);
        for i in 0..4 {
            cache.insert(i, i);
        }
        cache.set_capacity(2);
        assert_eq!(cache.get_size(), 2);
        assert!(cache.has(&3));
        assert!(cache.has(&2));
        assert!(!cache.has(&0));

        // A zero capacity ignores set_capacity (the engine's guard).
        let mut zero: LruCache<i32, i32> = LruCache::new(0);
        zero.insert(1, 1);
        assert_eq!(zero.get_size(), 0);
        zero.set_capacity(8);
        assert_eq!(zero.get_capacity(), 0);
    }

    #[test]
    fn peek_does_not_touch_the_order() {
        let mut cache: LruCache<i32, i32> = LruCache::new(2);
        cache.insert(1, 10);
        cache.insert(2, 20);
        assert_eq!(cache.peek(&1), Some(&10));
        // Peeking does not reorder, so 1 is still the oldest and is evicted.
        cache.insert(3, 30);
        assert!(!cache.has(&1));
        assert!(cache.has(&2));
        assert!(cache.has(&3));
    }
}
