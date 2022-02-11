use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

struct ConcurrentCache<K, V> {
    map: Arc<RwLock<HashMap<K, V>>>
}

impl <K, V> Clone for ConcurrentCache<K, V> {
    fn clone(&self) -> Self {
        ConcurrentCache { map: Arc::clone(&self.map) }
    }
}

impl<K, V> ConcurrentCache<K, V> where K: Hash + Eq + Copy, V: Copy {
    fn new() -> ConcurrentCache<K, V> {
        ConcurrentCache { map: Arc::new(RwLock::new(HashMap::new())) }
    }

    fn getOrLoad<F>(&self, key: K,  f: F) -> V where F: FnOnce(K) -> V {
        if let Some(value) = self.get(key) {
            return value;
        }
        let value = f(key);
        self.put(key, value);
        value
    }

    fn put(&self, key: K,  value: V) {
        let mut map_guard = self.map.write().unwrap();
        map_guard.insert(key, value);
    }

    fn get(&self, key: K) -> Option<V> {
        let mut map_guard = self.map.read().unwrap();
        map_guard.get(&key).map(|&value| value)
    }

    fn len(&self) -> usize {
        let mut map_guard = self.map.write().unwrap();
        map_guard.len()
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::thread::JoinHandle;
    use std::time::{Duration, Instant};
    use super::*;

    #[test]
    fn test_reads() {
        let mut cache = ConcurrentCache::new();
        cache.put(0, 100);
        cache.put(1, 100);

        let start = Instant::now();
        let tasks: Vec<JoinHandle<()>> = (0..4).map(|idx| (idx, cache.clone()))
            .map(|(idx, cache)| {
                thread::spawn(move || {
                    for _ in 0..100 {
                        cache.get(idx);
                        thread::sleep(Duration::from_millis(1));
                    }
                })
            }).collect();

        tasks.into_iter().for_each(|thread| {
            thread.join();
        });
        // all threads should read at the same time
        assert_eq!(start.elapsed().as_millis() < 150, true);
    }

    #[test]
    fn test_writes() {
        let mut cache = ConcurrentCache::new();
        let start = Instant::now();
        let tasks: Vec<JoinHandle<()>> = (0..4).map(|idx| (idx, cache.clone()))
            .map(|(idx, cache)| {
                thread::spawn(move || {
                    for i in 0..100 {
                        cache.put(idx * 100 + i, 100);
                    }
                })
            }).collect();

        tasks.into_iter().for_each(|thread| {
            thread.join();
        });
        assert_eq!(cache.len(), 4 * 100);
    }

    #[test]
    fn test_load() {
        let mut cache = ConcurrentCache::new();
        let start = Instant::now();
        let tasks: Vec<JoinHandle<()>> = (0..4).map(|idx| (idx, cache.clone()))
            .map(|(idx, cache)| {
                thread::spawn(move || {
                    for i in 0..100 {
                        cache.getOrLoad(idx * 100 + i, |x| { 100 });
                    }
                })
            }).collect();

        tasks.into_iter().for_each(|thread| {
            thread.join();
        });
        assert_eq!(cache.len(), 4 * 100);
    }

    #[test]
    fn test_reads_writes() {
        let mut cache = ConcurrentCache::new();
        let start = Instant::now();
        let tasks: Vec<JoinHandle<()>> = (0..4).map(|idx| (idx, cache.clone()))
            .map(|(idx, cache)| {
                if idx % 2 == 0 {
                    thread::spawn(move || {
                        for i in 0..100 {
                            cache.getOrLoad(idx * 100 + i, |x| { 100 });
                        }
                    })
                } else {
                    thread::spawn(move || {
                        for i in 0..100 {
                            cache.put(idx * 100 + i, 100);
                        }
                    })
                }
            }).collect();

        tasks.into_iter().for_each(|thread| {
            thread.join();
        });
        assert_eq!(cache.len(), 4 * 100);
    }

}
