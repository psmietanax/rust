use std::collections::HashMap;
use linked_hash_map::LinkedHashMap;

struct LRUCache<K, V> {
    map: LinkedHashMap<K, V>,
    capacity: usize
}

impl<K, V> LRUCache<K, V>
    where K: Hash + Eq {

    fn new(capacity: usize) -> Self {
        LRUCache { map: LinkedHashMap::new(), capacity }
    }

    fn put(&mut self, key: K, value: V) {
        self.map.insert(key, value);
        if self.map.len() > self.capacity {
            // evict
            self.map.pop_front();
        }
    }

    fn get(&mut self, key: K) -> Option<&V> {
        self.map.get_refresh(&key)
            .map(|x| &*x)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_value_update() {
        let mut cache = LRUCache::new(2);
        cache.put(1, 1);
        cache.put(1, 2);
        assert_eq!(cache.get(1), Some(&2));
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = LRUCache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        cache.put(3, 3);
        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), Some(&2));
        assert_eq!(cache.get(3), Some(&3));
    }

    #[test]
    fn test_lru_eviction_value_update() {
        let mut cache = LRUCache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        // move 1 to back
        cache.get(1);
        cache.put(3, 3);
        assert_eq!(cache.get(1), Some(&1));
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), Some(&3));
    }

}
