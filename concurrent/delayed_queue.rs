use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Condvar, LockResult, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic;
use std::thread;
use std::thread::ThreadId;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static SEQ_GENERATOR: AtomicUsize = AtomicUsize::new(0);

pub struct ScheduledEntry<T> {
    entry: T,
    id: u128,
    time: SystemTime
}

impl <T> ScheduledEntry<T> {
    pub fn of(entry: T, time: SystemTime) -> Self {
        ScheduledEntry {
            entry,
            time,
            id: SEQ_GENERATOR.fetch_add(1, atomic::Ordering::Release) as u128
        }
    }
    fn delay_ms(&self) -> i128 {
        self.time.duration_since(UNIX_EPOCH).unwrap().as_millis() as i128 -
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i128
    }
    pub fn entry(self) -> T {
        self.entry
    }
}

impl <T> Eq for ScheduledEntry<T> {}

impl <T> PartialEq<Self> for ScheduledEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        other.time.eq(&self.time) &&
            other.id.eq(&self.id)
    }
}

impl <T> PartialOrd<Self> for ScheduledEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut cmp = other.time.partial_cmp(&self.time);
        if cmp == Some(Ordering::Equal) {
            cmp = other.id.partial_cmp(&self.id)
        }
        cmp
    }
}

impl <T> Ord for ScheduledEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut cmp = other.time.cmp(&self.time);
        if cmp == Ordering::Equal {
            cmp = other.id.cmp(&self.id)
        }
        cmp
    }
}

struct SharedData<T> {
    queue: BinaryHeap<ScheduledEntry<T>>,
    leader: Option<ThreadId>
}

pub struct DelayedQueue<T> {
    data: Arc<Mutex<SharedData<T>>>,
    condvar: Arc<Condvar>,
    is_active: Arc<AtomicBool>
}

impl <T> Clone for DelayedQueue<T> {
    fn clone(&self) -> Self {
        DelayedQueue {
            data: Arc::clone(&self.data),
            condvar: Arc::clone(&self.condvar),
            is_active: Arc::clone(&self.is_active)
        }
    }
}

/*
This implementation uses the leader/follower pattern to efficiently minimize unnecessary timed waiting.
The leader will only wait for the delay time of the peek element in the queue, while other threads need to wait until signaled.
 */
impl <T> DelayedQueue<T> {
    pub fn new() -> Self {
        DelayedQueue {
            data: Arc::new(Mutex::new(
                SharedData {
                    queue: BinaryHeap::new(),
                    leader: None
                }
            )),
            condvar: Arc::new(Condvar::new()),
            is_active: Arc::new(AtomicBool::new(true))
        }
    }
    pub fn offer(&self, entry: ScheduledEntry<T>) {
        if self.is_active.load(atomic::Ordering::Acquire) {
            let mut guard = self.data.lock().unwrap();
            let entry_id = entry.id;
            guard.queue.push(entry);
            let first = guard.queue.peek();
            // if peek element has changed, update the leader
            // as the new element (peek element) will be available first
            // notify a new thread so it can become the leader
            if let Some(&ScheduledEntry { id, .. }) = first {
                if id == entry_id {
                    guard.leader.take();
                    self.condvar.notify_one();
                }
            }
        } else {
            panic!("Queue is not active");
        }
    }
    pub fn get(&self) -> Option<ScheduledEntry<T>> {
        if self.is_active.load(atomic::Ordering::Acquire) {
            let mut guard = self.data.lock().unwrap();
            let mut entry: Option<ScheduledEntry<T>> = None;
            loop {
                let first = guard.queue.peek();
                match first {
                    Some(scheduled_entry) => {
                        let delay = scheduled_entry.delay_ms();
                        // if an element is ready to be delivered, return it
                        // waiting is not required
                        if delay <= 0 {
                            entry = guard.queue.pop();
                            break;
                        }
                        // the leader is already waiting for the peek element, so wait for next turn
                        if guard.leader.is_some() {
                            guard = self.condvar.wait(guard).unwrap();
                            // there is no leader, so nominate the current thread to become the leader
                            // wait for the peek element scheduled delay
                        } else {
                            guard.leader.insert(thread::current().id());
                            guard = self.condvar.wait_timeout(guard, Duration::from_millis(delay as u64)).unwrap().0;
                            // if the current thread is still the leader, clean up and try to return the peek element
                            if let Some(leader) = guard.leader {
                                if leader == thread::current().id() {
                                    guard.leader.take();
                                }
                            }
                        }
                    },
                    None => {
                        // no peek element found, wait for one
                        guard = self.condvar.wait(guard).unwrap();
                    }
                }
                if !self.is_active.load(atomic::Ordering::Acquire) {
                    break;
                }
            }
            // wake up and nominate new leader if there is another peek element
            let first = guard.queue.peek();
            if guard.leader.is_none() && first.is_some() {
                self.condvar.notify_one();
            }
            // return the peek element
            entry
        } else {
            panic!("Queue is not active");
        }
    }

    pub fn stop(&self) {
        if self.is_active.compare_exchange(true, false, atomic::Ordering::Release, atomic::Ordering::Relaxed).is_ok() {
            self.condvar.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;
    use std::thread::JoinHandle;
    use futures::future::Join;
    use super::*;

    #[test]
    fn test_ascending() {
        let queue = DelayedQueue::new();
        let queue2 = queue.clone();
        let mut data = Vec::new();

        // add entries in ascending scheduled order
        let t = thread::spawn(move || {
            for i in 1..10 {
                queue2.offer(ScheduledEntry { entry: i * 10, id: i, time: SystemTime::now().add(Duration::from_millis(i as u64 * 10)) });
            }
        });

        for i in 1..10 {
            data.push(queue.get().unwrap().id);
        }

        assert_eq!(data, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_descending() {
        let queue = DelayedQueue::new();
        let queue2 = queue.clone();
        let mut data = Vec::new();

        // add entries in descending scheduled order
        let t = thread::spawn(move || {
            for i in 1..10 {
                queue2.offer(ScheduledEntry { entry: i * 10, id: i, time: SystemTime::now().add(Duration::from_millis(200 - i as u64 * 10)) });
            }
        });

        for i in 1..10 {
            data.push(queue.get().unwrap().id);
        }

        assert_eq!(data, vec![9, 8, 7, 6, 5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_multiple_writers() {
        let queue = DelayedQueue::new();
        let queue2 = queue.clone();
        let data = Arc::new(Mutex::new(Vec::new()));
        let data2 = Arc::clone(&data);

        // this entry should be available after 20ms
        queue.offer(ScheduledEntry { entry: 20, id: 2, time: SystemTime::now().add(Duration::from_millis(20)) });

        let t = thread::spawn(move || {
            data2.lock().unwrap().push(queue2.get().unwrap().id);
        });

        // this entry should be available after 10ms
        queue.offer(ScheduledEntry { entry: 10, id: 1, time: SystemTime::now().add(Duration::from_millis(10)) });

        data.lock().unwrap().push(queue.get().unwrap().id);
        t.join();

        assert_eq!(*data.lock().unwrap(), vec![1, 2]);
    }

    #[test]
    fn test_multiple_writers_and_readers() {
        let queue = DelayedQueue::new();
        let mut data = Arc::new(Mutex::new(Vec::new()));

        // 4 writers
        let r_threads: Vec<JoinHandle<()>> = (0..4).map(|idx| {
            let q = queue.clone();
            thread::spawn(move || {
                for i in 0..10 {
                    let num = 10 * idx + i;
                    q.offer(ScheduledEntry { entry: num, id: num, time: SystemTime::now().add(Duration::from_millis(200 - num as u64 )) });
                }
            })
        }).collect();

        // 4 readers
        let w_threads: Vec<JoinHandle<()>> = (0..4).map(|idx| {
            let d = Arc::clone(&data);
            let q = queue.clone();
            thread::spawn(move || {
                for i in 0..10 {
                    let elem = q.get().unwrap();
                    d.lock().unwrap().push(elem.id);
                }
            })
        }).collect();

        r_threads.into_iter().for_each(|handle| {
            handle.join();
        });
        w_threads.into_iter().for_each(|handle| {
            handle.join();
        });

        let expected: Vec<u128> = (0..40).rev().collect();
        assert_eq!(*data.lock().unwrap(), expected);
    }

    #[test]
    #[should_panic(expected = "Queue is not active")]
    fn test_stop() {
        let queue = DelayedQueue::new();
        for i in 0..10 {
            queue.offer(ScheduledEntry { entry: i, id: i, time: SystemTime::now() });
        }

        queue.stop();
        queue.get();
    }

}
