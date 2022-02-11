use std::hash::Hash;
use std::fmt::Debug;
use std::collections::HashMap;
use std::cmp::Ordering;

struct LFUCache<K, V> {
    data: Vec<Entry<K, V>>,
    capacity: usize,
    indexes: HashMap<K, usize>
}

#[derive(Debug)]
struct Entry<K, V> {
    key: K,
    value: V,
    freq: usize
}

impl <K, V> Entry<K, V> {
    fn new(key: K, value: V) -> Self {
        Self { key, value, freq: 0 }
    }
    fn increment(&mut self) {
        self.freq += 1;
    }
}

impl <K, V> PartialEq<Self> for Entry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
    }
}

impl <K, V> PartialOrd<Self> for Entry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl <K, V> Eq for Entry<K, V> {}

impl <K, V> Ord for Entry<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.freq.cmp(&other.freq)
    }
}

// A LFU cache backed by Binary Heap.
impl <K, V> LFUCache<K, V> where K: Ord + Debug + Hash + Copy, V: Copy + Debug + PartialEq {
    fn new(capacity: usize) -> Self {
        Self { data: Vec::new(), capacity, indexes: HashMap::new() }
    }

    fn put(&mut self, key: K, value: V) {
        if let Some(&idx) = self.indexes.get(&key) {
            let entry= &self.data[idx];
            if entry.value != value {
                self.del(idx);
                self.put_entry(Entry::new(key, value));
            }
        } else {
            self.put_entry(Entry::new(key, value));
        }
    }

    fn get(&mut self, key: K) -> Option<V> {
        if let Some(&idx) = self.indexes.get(&key) {
            let mut entry = self.del(idx).unwrap();
            let value = entry.value.clone();
            entry.increment();
            self.put_entry(entry);
            return Some(value);
        }
        None
    }

    fn put_entry(&mut self, entry: Entry<K, V>) {
        if self.data.len() == self.capacity {
            // remove the least frequent entry
            self.del(0).unwrap();
        }
        self.indexes.insert(entry.key.clone(), self.data.len());
        self.data.push(entry);
        self.heapify_bottom_up(self.data.len() - 1);
    }

    fn del(&mut self, idx: usize) -> Option<Entry<K, V>> {
        let len = self.data.len();
        if len > 0 && idx < len {
            self.data.swap(idx, len - 1);
            let entry = self.data.pop().unwrap();
            self.indexes.remove(&entry.key);
            if self.data.len() > 0 {
                self.indexes.insert(self.data[0].key, 0);
            }
            self.heapify_top_down(idx);
            Some(entry)
        } else {
            None
        }
    }

    fn heapify_bottom_up(&mut self, mut position: usize) {
        if position > 0 {
            let parent_position = (position - 1) / 2;
            if self.data[position].cmp(&self.data[parent_position]) == Ordering::Less {
                self.data.swap(position, parent_position);
                self.indexes.insert(self.data[position].key.clone(), position);
                self.indexes.insert(self.data[parent_position].key.clone(), parent_position);
                self.heapify_bottom_up(parent_position);
            }
        }
    }

    fn heapify_top_down(&mut self, mut position: usize) {
        let left = position * 2 + 1;
        let right = position * 2 + 2;
        if right < self.data.len()
            && self.data[right].cmp(&self.data[left]) == Ordering::Less
            && self.data[right].cmp(&self.data[position]) == Ordering::Less {
                self.data.swap(position, right);
                self.indexes.insert(self.data[position].key.clone(), position);
                self.indexes.insert(self.data[right].key.clone(), right);
                self.heapify_top_down(right);
        } else if left < self.data.len()
            && self.data[left].cmp(&self.data[position]) == Ordering::Less {
            self.data.swap(position, left);
            self.indexes.insert(self.data[position].key.clone(), position);
            self.indexes.insert(self.data[left].key.clone(), left);
            self.heapify_top_down(left);
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfu_value_update() {
        let mut cache = LFUCache::new(2);
        cache.put(1, 1);
        cache.put(1, 2);
        assert_eq!(cache.get(1), Some(2));
    }

    #[test]
    fn test_lfu_eviction() {
        let mut cache = LFUCache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        cache.put(3, 3);
        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), Some(2));
        assert_eq!(cache.get(3), Some(3));
    }

    #[test]
    fn test_lfu_eviction_value_update() {
        let mut cache = LFUCache::new(2);
        cache.put(1, 1);
        // increase freq counter for the key `1`
        cache.get(1);
        cache.put(2, 2);
        cache.put(3, 3);
        assert_eq!(cache.get(1), Some(1));
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), Some(3));
    }

}
