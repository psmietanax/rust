use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

struct CountdownLatch {
    guard: Mutex<usize>,
    cond: Condvar
}

impl CountdownLatch {
    fn new(counter: usize) -> Self {
        CountdownLatch { guard: Mutex::new(counter), cond: Condvar::new() }
    }
    fn countdown(&self) {
        let mut counter= self.guard.lock().unwrap();
        *counter -= 1;
        if *counter == 0 {
            self.cond.notify_all();
        }
    }
    fn wait(&self) {
        self.cond.wait(self.guard.lock().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use super::*;

    #[test]
    fn test_countdownlatch() {
        let latch = Arc::new(CountdownLatch::new(3));
        let latch_t1 = Arc::clone(&latch);
        let latch_t2 = Arc::clone(&latch);
        let latch_t3 = Arc::clone(&latch);

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            latch_t1.countdown();
        });

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            latch_t2.countdown();
        });

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            latch_t3.countdown();
        });

        let start = Instant::now();
        latch.wait();

        // Elapsed time must be > 100ms
        assert_eq!(start.elapsed().as_millis() > 100, true);
        // Counter has to be set to 0
        assert_eq!(*latch.guard.lock().unwrap(), 0);
    }

}
