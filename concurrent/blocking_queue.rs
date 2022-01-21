use std::collections::LinkedList;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

struct BlockingQueue<V> {
    data: Arc<SharedData<V>>
}

struct SharedData<V> {
    guard: Mutex<LinkedList<V>>,
    cond: Condvar,
    capacity: usize
}

impl <V> BlockingQueue<V> {
    fn new(capacity: usize) -> Self {
        BlockingQueue {
            data: Arc::new(SharedData {
                guard: Mutex::new(LinkedList::new()),
                cond: Condvar::new(),
                capacity
            })
        }
    }
    // blocking
    fn put(&mut self, v: V) {
        let mut linkedlist = self.data.guard.lock().unwrap();
        while linkedlist.len() == self.data.capacity {
            linkedlist = self.data.cond.wait(linkedlist).unwrap();
        }
        linkedlist.push_front(v);
        self.data.cond.notify_all();
    }
    fn take(&mut self) -> V {
        let mut linkedlist = self.data.guard.lock().unwrap();
        while linkedlist.len() == 0 {
            linkedlist = self.data.cond.wait(linkedlist).unwrap();
        }
        let result = linkedlist.pop_back().unwrap();
        self.data.cond.notify_all();
        result
    }
    // non-blocking
    fn offer(&self, v: V) -> bool {
        let mut linkedlist = self.data.guard.lock().unwrap();
        if linkedlist.len() < self.data.capacity {
            linkedlist.push_front(v);
            true
        } else {
            false
        }
    }
    fn poll(&self) -> Option<V> {
        self.data.guard.lock().unwrap().pop_back()
    }
}

impl <V> Clone for BlockingQueue<V> {
    fn clone(&self) -> Self {
        BlockingQueue {
            data: Arc::clone(&self.data)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use super::*;

    #[test]
    fn test_blocking() {
        let mut queue = BlockingQueue::new(2);
        let mut queue2 = queue.clone();

        thread::spawn(move || {
            for i in 0..10 {
                thread::sleep(Duration::from_millis(10));
                queue2.put(i);
            }
        });

        let start = Instant::now();
        let values: Vec<i32> = (0..10).map(|_| queue.take()).collect();

        // Elapsed time must be > 100ms (10 * 10ms)
        assert_eq!(start.elapsed().as_millis() > 100, true);
        assert_eq!(values, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_non_blocking() {
        let mut queue = BlockingQueue::new(2);
        let mut queue2 = queue.clone();

        // only 2 elements can be offered since the other thread is not consuming
        thread::spawn(move || {
            for i in 0..10 {
                queue2.offer(i);
            }
        });

        // wait for offer to complete and only then consume
        thread::sleep(Duration::from_millis(10));

        let start = Instant::now();
        let values: Vec<Option<i32>> = (0..10).map(|_| queue.poll()).collect();

        // Elapsed time must be < 100ms since it's non-blocking
        assert_eq!(start.elapsed().as_millis() < 100, true);
        assert_eq!(values, vec![Some(0), Some(1), None, None, None, None, None, None, None, None]);
    }

}
