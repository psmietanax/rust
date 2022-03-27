use std::thread;
use std::ops::{AddAssign, Div};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::{Local, DateTime};

static SECS: u64 = 5;
static MAX_REQS: u32 = 100;
static REFRESH_SECS: u64 = 1;

fn main() {
    let lb = LeakyBucket::new(MAX_REQS, Duration::from_secs(REFRESH_SECS));
    let result = Arc::new(Mutex::new(Vec::new()));

    (0..4).for_each(|_| {
        let lbb = lb.clone();
        let ret = result.clone();
        thread::spawn(move || {
            loop {
                if lbb.acquire() {
                    ret.lock().unwrap().push(1);
                }
            }
        });
    });

    thread::sleep(Duration::from_secs(SECS));

    let counter = result.lock().unwrap().iter().count();

    println!("PERMITS: {}, allowed ~ {}", counter, MAX_REQS as u64 * SECS);
}

struct LeakyBucket {
    next_available_time: Arc<Mutex<Instant>>,
    wait_interval: Duration
}

impl Clone for LeakyBucket {
    fn clone(&self) -> Self {
        LeakyBucket {
            next_available_time: Arc::clone(&self.next_available_time),
            wait_interval: self.wait_interval.clone()
        }
    }
}

impl LeakyBucket {
    fn acquire(&self) -> bool {
        let mut next_available_time = self.next_available_time.lock().unwrap();
        if Instant::now() >= *next_available_time {
            next_available_time.add_assign(self.wait_interval);
            true
        } else {
            false
        }
    }
}

impl LeakyBucket {
    fn new(max_request: u32, refresh_interval: Duration) -> Self {
        let wait_interval = refresh_interval.div(max_request);
        LeakyBucket {
            next_available_time: Arc::new(Mutex::new(Instant::now())),
            wait_interval,
        }
    }
}
