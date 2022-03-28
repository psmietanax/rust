use std::thread;
use std::ops::{AddAssign, Div};
use std::sync::{Arc, Mutex, Condvar};
use std::time::{Duration, Instant};
use chrono::{Local, DateTime};

static SECS: u64 = 5;
static MAX_REQS: usize = 1000;
static REFRESH_SECS: u64 = 1;
static BUCKET_SIZE: usize = 5;
static THREAD_WAIT_MILLIS: u64 = 100;

fn main() {
    /*let lb = LeakyBucket::new(MAX_REQS, Duration::from_secs(REFRESH_SECS), BUCKET_SIZE, Duration::from_millis(THREAD_WAIT_MILLIS));
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

    println!("PERMITS: {}, allowed ~ {}", counter, MAX_REQS as u64 * SECS);*/

    //

    let lb = TokenBucket::new(MAX_REQS, Duration::from_secs(REFRESH_SECS));
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

    println!("PERMITS: {}, allowed ~ {}", counter, MAX_REQS as u64 * SECS.div(REFRESH_SECS));

}

struct LeakyBucketData {
    next_available_time: Instant,
    curr_buffer_size: usize
}

pub struct LeakyBucket {
    data: Arc<Mutex<LeakyBucketData>>,
    condvar: Arc<Condvar>,
    wait_interval: Duration,
    thread_wait_timeout: Duration,
    buffer_size: usize
}

impl Clone for LeakyBucket {
    fn clone(&self) -> Self {
        LeakyBucket {
            data: Arc::clone(&self.data),
            condvar: Arc::clone(&self.condvar),
            wait_interval: self.wait_interval.clone(),
            thread_wait_timeout: self.thread_wait_timeout.clone(),
            buffer_size: self.buffer_size.clone()
        }
    }
}

impl LeakyBucket {
    pub fn new(max_request_per_interval: usize, refresh_interval: Duration, buffer_size: usize, thread_wait_timeout: Duration) -> Self {
        assert!(max_request_per_interval > 0);

        let wait_interval = refresh_interval.div(max_request_per_interval as u32);
        LeakyBucket {
            data: Arc::new(Mutex::new(LeakyBucketData {
                next_available_time: Instant::now(),
                curr_buffer_size: 0
            })),
            condvar: Arc::new(Condvar::new()),
            wait_interval,
            thread_wait_timeout,
            buffer_size
        }
    }
    pub fn acquire(&self) -> bool {
        let mut data = self.data.lock().unwrap();

        let mut thread_wait_timeout = self.thread_wait_timeout;
        loop {
            if Instant::now() >= data.next_available_time {
                data.next_available_time.add_assign(self.wait_interval);
                return true;
            } else if data.curr_buffer_size < self.buffer_size && !thread_wait_timeout.is_zero() {
                data.curr_buffer_size += 1;
                let wait_time = (data.next_available_time.saturating_duration_since(Instant::now())).min(thread_wait_timeout);
                thread_wait_timeout = thread_wait_timeout.saturating_sub(wait_time);
                data = self.condvar.wait_timeout(data, wait_time).unwrap().0;
                data.curr_buffer_size -= 1;
            } else {
                return false;
            }
        }
    }
}

//

struct TokenBucketData {
    next_refill_time: Instant,
    tokens_count: usize
}

pub struct TokenBucket {
    data: Arc<Mutex<TokenBucketData>>,
    condvar: Arc<Condvar>,
    tokens_per_interval: usize,
    refill_interval: Duration
}

impl Clone for TokenBucket {
    fn clone(&self) -> Self {
        TokenBucket {
            data: Arc::clone(&self.data),
            condvar: Arc::clone(&self.condvar),
            tokens_per_interval: self.tokens_per_interval.clone(),
            refill_interval: self.refill_interval.clone()
        }
    }
}

impl TokenBucket {
    pub fn new(tokens_per_interval: usize, refill_interval: Duration) -> Self {
        assert!(tokens_per_interval > 0);

        TokenBucket {
            data: Arc::new(Mutex::new(TokenBucketData {
                next_refill_time: Instant::now(),
                tokens_count: 0
            })),
            condvar: Arc::new(Condvar::new()),
            tokens_per_interval,
            refill_interval
        }
    }
    pub fn acquire(&self) -> bool {
        let mut data = self.data.lock().unwrap();

        if Instant::now() > data.next_refill_time {
            // refill
            data.tokens_count = self.tokens_per_interval;
            data.next_refill_time += self.refill_interval;
        }

        if data.tokens_count > 0 {
            data.tokens_count -= 1;
            true
        } else {
            false
        }
    }
}

// add fixed-window
// add sliding-window
