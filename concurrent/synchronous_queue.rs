use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicBool};
use crossbeam::atomic::AtomicCell;
use std::hint::spin_loop;
use std::thread;

struct SynchronousQueue<T> {
    is_sender_present: Arc<AtomicBool>,
    is_receiver_present: Arc<AtomicBool>,
    exchanger: Arc<AtomicCell<Option<T>>>
}

impl <T> Clone for SynchronousQueue<T> {
    fn clone(&self) -> Self {
        SynchronousQueue {
            exchanger: Arc::clone(&self.exchanger),
            is_sender_present: Arc::clone(&self.is_sender_present),
            is_receiver_present: Arc::clone(&self.is_receiver_present)
        }
    }
}

impl <T> SynchronousQueue<T> where T: Copy + Eq {
    fn new() -> Self {
        SynchronousQueue {
            exchanger: Arc::new(AtomicCell::new(None)),
            is_sender_present: Arc::new(AtomicBool::new(false)),
            is_receiver_present: Arc::new(AtomicBool::new(false))
        }
    }
    fn put(&self, value: T) {
        // acquire the sender slot
        while self.is_sender_present.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire).is_err() {
            spin_loop();
        }
        // waiting for a receiver to be present
        while !self.is_receiver_present.load(Ordering::Acquire) {
            spin_loop();
        }
        // set the value
        self.exchanger.store(Some(value));
        // waiting for a receiver to complete
        while self.exchanger.load().is_some() {
            spin_loop();
        }
        // release the sender slot
        self.is_sender_present.store(false, Ordering::Release);
    }
    fn take(&self) -> T {
        // acquire the receiver slot
        while self.is_receiver_present.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire).is_err() {
            spin_loop();
        }
        // waiting for a sender to be present
        while !self.is_sender_present.load(Ordering::Acquire) {
            spin_loop();
        }
        // waiting for a value
        while self.exchanger.load().is_none() {
            spin_loop();
        }
        // get a value
        let value = self.exchanger.swap(None).unwrap();
        // release the receiver slot
        self.is_receiver_present.store(false, Ordering::Release);
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::AsyncReadExt;

    #[test]
    fn test_synchronous_queue() {
        let sq_s = SynchronousQueue::new();
        let sq_r = sq_s.clone();

        thread::spawn(move || {
            for i in 0..10 {
                sq_s.put(i);
            }
        });

        let mut result = Vec::with_capacity(10);
        for _ in 0..10 {
            result.push(sq_r.take());
        }
        assert_eq!(result, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_synchronous_queue_single_writer_multiple_readers() {
        let sq_s = SynchronousQueue::new();
        let sq_r1 = sq_s.clone();
        let sq_r2 = sq_s.clone();

        let t1 = thread::spawn(move || {
            let mut result = Vec::new();
            for _ in 0..5 {
                result.push(sq_r1.take());
            }
            result
        });

        let t2 = thread::spawn(move || {
            let mut result = Vec::new();
            for _ in 0..5 {
                result.push(sq_r2.take());
            }
            result
        });

        for i in 0..10 {
            sq_s.put(i);
        }

        let mut result = Vec::with_capacity(8);
        result.extend(t1.join().unwrap());
        result.extend(t2.join().unwrap());
        result.sort();

        assert_eq!(result, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_synchronous_queue_single_reader_multiple_writers() {
        let sq_r = SynchronousQueue::new();
        let sq_s1 = sq_r.clone();
        let sq_s2 = sq_r.clone();

        thread::spawn(move || {
            for i in 0..5 {
                sq_s1.put(i);
            }
        });

        thread::spawn(move || {
            for i in 5..10 {
                sq_s2.put(i);
            }
        });

        let mut result = Vec::with_capacity(10);
        for _ in 0..10 {
            result.push(sq_r.take());
        }
        result.sort();

        assert_eq!(result, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
