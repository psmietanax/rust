use std::{mem, thread};
use std::ops::{Add, Div};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn main() {
    LeakyBucket::new(100, Duration::from_secs(10), Duration::from_secs(1));
}

struct LeakyBucket {
    next_available_time: Arc<Mutex<Instant>>,
    wait_interval: Duration,
    timeout: Duration
}

impl LeakyBucket {
    fn acquire(&self) -> bool {
        {
            let next_available_time = self.next_available_time.lock().unwrap();
            if Instant::now() >= *next_available_time {
                print!("yeah");
                *next_available_time.add(self.wait_interval);
            } else {
                mem::drop(next_available_time);
                thread::park_timeout()
            }
        }
        true

    }
}

impl LeakyBucket {
    fn new(max_request: u32, refresh_interval: Duration, timeout: Duration) -> Self {
        let wait_interval = refresh_interval.div(max_request);
        LeakyBucket {
            next_available_time: Arc::new(Mutex::new(Instant::now())),
            wait_interval,
            timeout
        }
    }
}
