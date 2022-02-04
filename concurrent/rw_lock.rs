use std::sync::{Condvar, Arc, Mutex};
use std::borrow::Borrow;
use std::thread;
use std::time::Duration;
use crate::rw_lock::LockType::{READ, WRITE};

struct RWLock {
    r_lock: Lock,
    w_lock: Lock
}

impl RWLock {
    fn new() -> Self {
        let guard = Arc::new((Mutex::new(0), Condvar::new()));
        Self {
            r_lock: Lock { lock: Arc::clone(&guard), kind: READ },
            w_lock: Lock { lock: guard, kind: WRITE }
        }
    }
    fn r_lock(&self) -> Lock {
        self.r_lock.clone()
    }

    fn w_lock(&self) -> Lock {
        self.w_lock.clone()
    }
}

#[derive(Clone)]
enum LockType {
    READ, WRITE
}

struct Lock {
    lock: Arc<(Mutex<i32>, Condvar)>,
    kind: LockType,
}

impl Clone for Lock {
    fn clone(&self) -> Self {
        Self { lock: Arc::clone(&self.lock), kind: LockType::from(self.kind.clone()) }
    }
}

impl Lock {
    fn lock(&self) {
        let (mutex, condvar) = self.lock.borrow();
        match self.kind {
            READ => {
                let mut locked_counter = mutex.lock().unwrap();
                if *locked_counter >= 0 {
                    *locked_counter += 1;
                } else {
                    while *locked_counter < 0 {
                        locked_counter = condvar.wait(locked_counter).unwrap();
                    }
                }
            },
            WRITE => {
                let mut locked_counter = mutex.lock().unwrap();
                if *locked_counter == 0 {
                    *locked_counter = -1;
                } else {
                    while *locked_counter != 0 {
                        locked_counter = condvar.wait(locked_counter).unwrap();
                    }
                }
            }
        }
    }

    fn unlock(&self) {
        let (mutex, condvar) = self.lock.borrow();
        match self.kind {
            READ => {
                let mut locked_counter = mutex.lock().unwrap();
                if *locked_counter > 0 {
                    *locked_counter -= 1;
                    if *locked_counter == 0 {
                        condvar.notify_one();
                    }
                } else {
                    panic!("Cannot unlock");
                }
            },
            WRITE => {
                let mut locked_counter = mutex.lock().unwrap();
                if *locked_counter == -1 {
                    *locked_counter = 0;
                    condvar.notify_one();
                } else {
                    panic!("Cannot unlock");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    #[test]
    fn test_read_lock() {
        // multiple must proceed while acquiring read lock
        let rwlock = RWLock::new();
        let r_lock1 = rwlock.r_lock();
        let r_lock2 = rwlock.r_lock();
        let r_lock3 = rwlock.r_lock();

        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::clone(&counter1);
        let counter3 = Arc::clone(&counter1);

        thread::spawn(move || {
            r_lock1.lock();
            counter1.fetch_add(1, Ordering::Acquire);
        });

        thread::spawn(move || {
            r_lock2.lock();
            counter2.fetch_add(1, Ordering::Acquire);
        });

        // allow other threads to acquire the lock
        thread::sleep(Duration::from_millis(10));
        r_lock3.lock();
        counter3.fetch_add(1, Ordering::Acquire);

        assert_eq!(counter3.fetch_add(0, Ordering::Acquire), 3);
    }

    #[test]
    fn test_write_lock() {
        // only one thread must proceed while acquiring write lock
        let rwlock = RWLock::new();
        let w_lock1 = rwlock.w_lock();
        let w_lock2 = rwlock.w_lock();

        thread::spawn(move || {
            w_lock1.lock();
            thread::sleep(Duration::from_millis(100));
            w_lock1.unlock();
        });

        // allow other threads to acquire the lock
        let start = Instant::now();
        thread::sleep(Duration::from_millis(10));
        w_lock2.lock();

        // main thread must wait till the first thread releases the lock
        assert_eq!(start.elapsed().as_millis() >= 100, true);
    }

    #[test]
    fn test_write_read_lock() {
        // only one thread must proceed while acquiring write lock
        let rwlock = RWLock::new();
        let w_lock1 = rwlock.w_lock();
        let r_lock1 = rwlock.r_lock();

        thread::spawn(move || {
            w_lock1.lock();
            thread::sleep(Duration::from_millis(100));
            w_lock1.unlock();
        });

        let start = Instant::now();
        // allow other threads to acquire the lock
        thread::sleep(Duration::from_millis(10));
        r_lock1.lock();

        // main thread must wait till the thread releases the write lock
        assert_eq!(start.elapsed().as_millis() >= 100, true);
    }

    #[test]
    fn test_read_write_lock() {
        let rwlock = RWLock::new();
        let w_lock1 = rwlock.w_lock();
        let r_lock1 = rwlock.r_lock();
        let r_lock2 = rwlock.r_lock();

        thread::spawn(move || {
            r_lock1.lock();
            thread::sleep(Duration::from_millis(50));
            r_lock1.unlock();
        });

        thread::spawn(move || {
            r_lock2.lock();
            thread::sleep(Duration::from_millis(100));
            r_lock2.unlock();
        });

        let start = Instant::now();
        // allow other threads to acquire the lock
        thread::sleep(Duration::from_millis(10));
        w_lock1.lock();

        // main thread must wait till the threads release the read lock
        assert_eq!(start.elapsed().as_millis() > 100, true);
    }

}
