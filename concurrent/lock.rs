use std::sync::{Arc, Condvar, Mutex};

struct Lock {
    lock: Arc<(Mutex<bool>, Condvar)>
}

impl Clone for Lock {
    fn clone(&self) -> Self {
        Self { lock: Arc::clone(&self.lock) }
    }
}

impl Lock {
    fn new() -> Self {
        Self { lock: Arc::new((Mutex::new(false), Condvar::new())) }
    }

    fn lock(&self) {
        let (mutex, condvar) = &*self.lock;
        let mut is_locked = mutex.lock().unwrap();
        // protection against spurious wakeups
        while *is_locked {
            is_locked = condvar.wait(is_locked).unwrap();
        }
        *is_locked = true;
    }

    fn unlock(&self) {
        let (mutex, condvar) = &*self.lock;
        let mut is_locked = mutex.lock().unwrap();
        if *is_locked {
            *is_locked = false;
            condvar.notify_one();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;
    use std::thread;
    use std::time::{Duration, Instant};
    use super::*;

    #[test]
    fn test_lock() {
        let lock = Lock::new();
        let lock2 = lock.clone();

        thread::spawn(move || {
            lock2.lock();
            thread::sleep(Duration::from_millis(100));
            lock2.unlock();
        });

        let start = Instant::now();
        // allow the thread above to acquire the lock
        thread::sleep(Duration::from_millis(10));
        // wait till the lock is released
        lock.lock();
        // Elapsed time must be > 100ms
        assert_eq!(start.elapsed().as_millis() > 100, true);
    }

    #[test]
    fn test_lock_multiple_threads() {
        let lock = Lock::new();
        let lock2 = lock.clone();
        let lock3 = lock.clone();

        thread::spawn(move || {
            lock2.lock();
            thread::sleep(Duration::from_millis(50));
            lock2.unlock();
        });

        thread::spawn(move || {
            lock3.lock();
            thread::sleep(Duration::from_millis(50));
            lock3.unlock();
        });

        let start = Instant::now();
        // allow the threads above to acquire the lock
        thread::sleep(Duration::from_millis(10));
        // wait till the lock is released
        lock.lock();
        // Elapsed time must be > 100ms
        assert_eq!(start.elapsed().as_millis() > 100, true);
    }
}
