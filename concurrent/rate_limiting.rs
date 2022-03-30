use std::thread;
use std::ops::{AddAssign, Div, Sub};
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::AtomicUsize;
use std::time::{Duration, Instant};

// Rate Limiting algorithms: LeakyBucket, TokenBucket, FixedWindow, SlidingWindow

struct LeakyBucketData {
    next_available_time: Instant,
    curr_buffer_size: usize
}

/*
   A leaky bucket algorithm to control the traffic rate.
   A bucket with a fixed buffer_size holds acquire requests up to the given wait_time.
   Requests are allowed to acquire max_requests_per_interval per refresh_interval evenly.
   The implementation is thread-safe.
 */
pub struct LeakyBucket {
    data: Arc<Mutex<LeakyBucketData>>,
    condvar: Arc<Condvar>,
    wait_interval: Duration,
    wait_timeout: Duration,
    buffer_size: usize
}

impl Clone for LeakyBucket {
    fn clone(&self) -> Self {
        LeakyBucket {
            data: Arc::clone(&self.data),
            condvar: Arc::clone(&self.condvar),
            wait_interval: self.wait_interval.clone(),
            wait_timeout: self.wait_timeout.clone(),
            buffer_size: self.buffer_size.clone()
        }
    }
}

impl LeakyBucket {
    pub fn new(max_requests_per_interval: usize, refresh_interval: Duration, buffer_size: usize, wait_timeout: Duration) -> Self {
        assert!(max_requests_per_interval > 0);

        let wait_interval = refresh_interval.div(max_requests_per_interval as u32);
        LeakyBucket {
            data: Arc::new(Mutex::new(LeakyBucketData {
                next_available_time: Instant::now(),
                curr_buffer_size: 0
            })),
            condvar: Arc::new(Condvar::new()),
            wait_interval,
            wait_timeout,
            buffer_size
        }
    }
    pub fn acquire(&self) -> bool {
        let mut data = self.data.lock().unwrap();

        let mut wait_timeout = self.wait_timeout;
        loop {
            let now = Instant::now();
            if now >= data.next_available_time {
                data.next_available_time.add_assign(self.wait_interval);
                return true;
            } else if data.curr_buffer_size < self.buffer_size && !wait_timeout.is_zero() {
                data.curr_buffer_size += 1;
                let wait_time = (data.next_available_time.saturating_duration_since(now)).min(wait_timeout);
                wait_timeout = wait_timeout.saturating_sub(wait_time);
                data = self.condvar.wait_timeout(data, wait_time).unwrap().0;
                data.curr_buffer_size -= 1;
            } else {
                return false;
            }
        }
    }
}

//

#[derive(Debug)]
struct TokenBucketData {
    next_refill_time: Instant,
    tokens_count: usize
}

/*
   A token bucket algorithm to control the traffic rate.
   A token bucket with a fixed buffer_size holds tokens.
   Tokens are refilled each refill_interval.
   Requests are allowed to acquire if a token for a given request is available.
   The implementation is thread-safe.
 */
pub struct TokenBucket {
    data: Arc<Mutex<TokenBucketData>>,
    condvar: Arc<Condvar>,
    tokens_per_interval: usize,
    refill_interval: Duration,
    buffer_size: usize
}

impl Clone for TokenBucket {
    fn clone(&self) -> Self {
        TokenBucket {
            data: Arc::clone(&self.data),
            condvar: Arc::clone(&self.condvar),
            tokens_per_interval: self.tokens_per_interval.clone(),
            refill_interval: self.refill_interval.clone(),
            buffer_size: self.buffer_size.clone()
        }
    }
}

impl TokenBucket {
    pub fn new(tokens_per_interval: usize, refill_interval: Duration, buffer_size: usize) -> Self {
        assert!(tokens_per_interval > 0);

        TokenBucket {
            data: Arc::new(Mutex::new(TokenBucketData {
                next_refill_time: Instant::now(),
                tokens_count: 0
            })),
            condvar: Arc::new(Condvar::new()),
            tokens_per_interval,
            refill_interval,
            buffer_size
        }
    }
    pub fn acquire(&self) -> bool {
        let mut data = self.data.lock().unwrap();

        if Instant::now() > data.next_refill_time {
            // refill
            data.tokens_count = (data.tokens_count + self.tokens_per_interval).min(self.buffer_size);
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

//

struct FixedWindowData {
    counter: usize,
    interval_start_time: Instant
}

/*
   A fixed window algorithm to control the traffic rate.
   The timeline is divided into fixed windows of refresh_interval and and each window is provided with a counter.
   Requests are allowed to acquire if a max_request_per_interval for a given window is not exceeded.
   The implementation is thread-safe.
 */
pub struct FixedWindow {
    data: Arc<Mutex<FixedWindowData>>,
    max_request_per_interval: usize,
    refresh_interval: Duration
}

impl Clone for FixedWindow {
    fn clone(&self) -> Self {
        FixedWindow {
            data: Arc::clone(&self.data),
            max_request_per_interval: self.max_request_per_interval.clone(),
            refresh_interval: self.refresh_interval.clone()
        }
    }
}

impl FixedWindow {
    pub fn new(max_request_per_interval: usize, refresh_interval: Duration) -> Self {
        assert!(max_request_per_interval > 0);

        FixedWindow {
            data: Arc::new(Mutex::new(FixedWindowData {
                counter: 0,
                interval_start_time: Instant::now()
            })),
            max_request_per_interval,
            refresh_interval
        }
    }
    pub fn acquire(&self) -> bool {
        let mut data = self.data.lock().unwrap();

        let now = Instant::now();

        // if we passed the current window, move to another window
        if now - data.interval_start_time > self.refresh_interval {
            data.counter = 1;
            data.interval_start_time = now;
            true
        } else if data.counter < self.max_request_per_interval {
            data.counter += 1;
            true
        } else {
            false
        }
    }
}

//

struct SlidingWindowData {
    counter: usize,
    prev_counter: usize,
    interval_start_time: Instant
}

/*
   A sliding window algorithm to control the traffic rate.
   The implementation is based on FixedWindow, but instead of restarting the counter after
   every window, the counter information from the previous window is used to estimate
   the size of the counter in the current window.
   Instead of fixed window size, there is a rolling window of time to smooth bursts.
   The implementation is thread-safe.
 */
pub struct SlidingWindow {
    data: Arc<Mutex<SlidingWindowData>>,
    max_request_per_interval: usize,
    interval: Duration
}

impl Clone for SlidingWindow {
    fn clone(&self) -> Self {
        SlidingWindow {
            data: Arc::clone(&self.data),
            max_request_per_interval: self.max_request_per_interval.clone(),
            interval: self.interval.clone()
        }
    }
}

impl SlidingWindow {
    pub fn new(max_request_per_interval: usize, interval: Duration) -> Self {
        assert!(max_request_per_interval > 0);

        SlidingWindow {
            data: Arc::new(Mutex::new(SlidingWindowData {
                counter: 0,
                prev_counter: max_request_per_interval,
                interval_start_time: Instant::now()
            })),
            max_request_per_interval,
            interval
        }
    }
    pub fn acquire(&self) -> bool {
        let mut data = self.data.lock().unwrap();

        let now = Instant::now();
        let since_last_interval = now.saturating_duration_since(data.interval_start_time);

        // if we passed the current window, move to another window
        if since_last_interval >= self.interval {
            data.prev_counter = data.counter;
            data.counter = 0;
            data.interval_start_time = now;
        }

        // counter ratio from the previous window
        let prev_window_counter_ratio = 1.0 - since_last_interval.as_nanos() as f32 / self.interval.as_nanos() as f32;
        // counter from the previous window based on the ratio
        let prev_window_counter = (prev_window_counter_ratio * data.prev_counter as f32) as usize;
        let counter = prev_window_counter + data.counter;

        if counter < self.max_request_per_interval {
            data.counter += 1;
            true
        } else {
            false
        }
    }
}

static RUNTIME_SECS: u64 = 5;
static MAX_REQS: usize = 1000;
static REFRESH_SECS: u64 = 1;
static BUFFER_SIZE: usize = 5;
static THREAD_WAIT_MILLIS: u64 = 100;

fn main() {
    let lb = LeakyBucket::new(MAX_REQS, Duration::from_secs(REFRESH_SECS), BUFFER_SIZE, Duration::from_millis(THREAD_WAIT_MILLIS));
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
    thread::sleep(Duration::from_secs(RUNTIME_SECS));
    let counter = result.lock().unwrap().iter().count();
    println!("LeakyBucket permits: {}, allowed ~ {}", counter, MAX_REQS as u64 * RUNTIME_SECS);

    //

    let tb = TokenBucket::new(MAX_REQS, Duration::from_secs(REFRESH_SECS), MAX_REQS);
    let result = Arc::new(Mutex::new(Vec::new()));

    (0..4).for_each(|_| {
        let lbb = tb.clone();
        let ret = result.clone();
        thread::spawn(move || {
            loop {
                if lbb.acquire() {
                    ret.lock().unwrap().push(1);
                }
            }
        });
    });

    thread::sleep(Duration::from_secs(RUNTIME_SECS));
    let counter = result.lock().unwrap().iter().count();
    println!("TokenBucket permits: {}, allowed ~ {}", counter, MAX_REQS as u64 * RUNTIME_SECS.div(REFRESH_SECS));

    //

    let fw = FixedWindow::new(MAX_REQS, Duration::from_secs(REFRESH_SECS));
    let result = Arc::new(Mutex::new(Vec::new()));

    (0..4).for_each(|_| {
        let lbb = fw.clone();
        let ret = result.clone();
        thread::spawn(move || {
            loop {
                if lbb.acquire() {
                    ret.lock().unwrap().push(1);
                }
            }
        });
    });

    thread::sleep(Duration::from_secs(RUNTIME_SECS));
    let counter = result.lock().unwrap().iter().count();
    println!("FixedWindow permits: {}, allowed ~ {}", counter, MAX_REQS as u64 * RUNTIME_SECS.div(REFRESH_SECS));

    //

    let sw = SlidingWindow::new(MAX_REQS, Duration::from_secs(REFRESH_SECS));
    let result = Arc::new(Mutex::new(Vec::new()));

    (0..4).for_each(|_| {
        let lbb = sw.clone();
        let ret = result.clone();
        thread::spawn(move || {
            loop {
                if lbb.acquire() {
                    ret.lock().unwrap().push(1);
                }
            }
        });
    });

    thread::sleep(Duration::from_secs(RUNTIME_SECS));
    let counter = result.lock().unwrap().iter().count();
    println!("SlidingWindow permits: {}, allowed ~ {}", counter, MAX_REQS as u64 * RUNTIME_SECS.div(REFRESH_SECS));
}

#[cfg(test)]
mod tests {
    use std::hint::spin_loop;
    use super::*;
    use std::sync::Barrier;
    use std::time::Instant;
    use std::thread;

    #[test]
    fn test_leaky_bucket_exceed_buffer_size() {
        let leaky_bucket = LeakyBucket::new(1, Duration::from_millis(10), 5, Duration::from_millis(100));
        (0..10).for_each(|_| {
            let lb = leaky_bucket.clone();
            thread::spawn(move || {
                lb.acquire();
            });
        });
        thread::sleep(Duration::from_millis(5));
        assert_eq!(leaky_bucket.acquire(), false);
    }

    #[test]
    fn test_leaky_bucket_timeout() {
        let leaky_bucket = LeakyBucket::new(1, Duration::from_millis(50), 5, Duration::from_millis(100));
        leaky_bucket.acquire();
        let start = Instant::now();
        assert!(leaky_bucket.acquire());
        // second request can wait up to 100ms
        assert!(start.elapsed().as_millis() >= 50);
    }

    #[test]
    fn test_leaky_bucket() {
        let leaky_bucket = LeakyBucket::new(10, Duration::from_millis(100), 10, Duration::from_millis(10));
        let counter =  Arc::new(AtomicUsize::new(0));
        let reqs_diff = Arc::new(Mutex::new((Vec::new(), Instant::now())));

        (0..4).for_each(|_| {
            let lb = leaky_bucket.clone();
            let c = counter.clone();
            let rd = reqs_diff.clone();
            thread::spawn(move || {
                loop {
                    if lb.acquire() {
                        c.fetch_add(1, Ordering::SeqCst);
                        let (ref mut vec, ref mut time) = *rd.lock().unwrap();
                        let duration = Instant::now().saturating_duration_since(*time).as_millis();
                        *time = Instant::now();
                        vec.push(duration);
                    }
                }
            });
        });

        let start = Instant::now();
        while counter.load(Ordering::SeqCst) < 500 {
            spin_loop();
        }

        // 10 reqs per 100 ms
        // to get 500 reqs, we have to spend 500/10 = 50, 50 * 100ms = 5000ms
        assert!(start.elapsed().as_millis() >= 4800 && start.elapsed().as_millis() <= 5200);
        // average difference between consecutive requests
        let avg = reqs_diff.lock().unwrap().0.iter().map(|x| x.clone() as f32).reduce(|x, y| (x + y) / 2.0).unwrap();
        assert!(avg > 9.0 && avg < 11.0);
    }

    #[test]
    fn test_token_bucket() {
        let token_bucket = TokenBucket::new(10, Duration::from_millis(100), 10);
        let counter =  Arc::new(AtomicUsize::new(0));
        let reqs_diff = Arc::new(Mutex::new((Vec::new(), Instant::now())));

        (0..4).for_each(|_| {
            let tb = token_bucket.clone();
            let c = counter.clone();
            let rd = reqs_diff.clone();
            thread::spawn(move || {
                loop {
                    if tb.acquire() {
                        c.fetch_add(1, Ordering::SeqCst);
                        let (ref mut vec, ref mut time) = *rd.lock().unwrap();
                        let duration = Instant::now().saturating_duration_since(*time).as_millis();
                        *time = Instant::now();
                        vec.push(duration);
                    }
                }
            });
        });

        let start = Instant::now();
        while counter.load(Ordering::SeqCst) < 500 {
            spin_loop();
        }

        // 10 reqs refilled each 100 ms
        // to get 500 reqs, we have to spend 500/10 = 50, 50 * 100ms = 5000ms
        assert!(start.elapsed().as_millis() >= 4800 && start.elapsed().as_millis() <= 5200);
        // average difference between consecutive requests
        let avg = reqs_diff.lock().unwrap().0.iter().map(|x| x.clone() as f32).reduce(|x, y| (x + y) / 2.0).unwrap();
        assert!(avg < 1.0);
    }

    #[test]
    fn test_fixed_window() {
        let fixed_window = FixedWindow::new(10, Duration::from_millis(100));
        let counter =  Arc::new(AtomicUsize::new(0));
        let reqs_diff = Arc::new(Mutex::new((Vec::new(), Instant::now())));

        (0..4).for_each(|_| {
            let fw = fixed_window.clone();
            let c = counter.clone();
            let rd = reqs_diff.clone();
            thread::spawn(move || {
                loop {
                    if fw.acquire() {
                        c.fetch_add(1, Ordering::SeqCst);
                        let (ref mut vec, ref mut time) = *rd.lock().unwrap();
                        let duration = Instant::now().saturating_duration_since(*time).as_millis();
                        *time = Instant::now();
                        vec.push(duration);
                    }
                }
            });
        });

        let start = Instant::now();
        while counter.load(Ordering::SeqCst) < 500 {
            spin_loop();
        }

        // 10 reqs refilled each 100 ms
        // to get 500 reqs, we have to spend 500/10 = 50, 50 * 100ms = 5000ms
        assert!(start.elapsed().as_millis() >= 4800 && start.elapsed().as_millis() <= 5200);
        // average difference between consecutive requests
        let avg = reqs_diff.lock().unwrap().0.iter().map(|x| x.clone() as f32).reduce(|x, y| (x + y) / 2.0).unwrap();
        assert!(avg < 1.0);
    }

    #[test]
    fn test_sliding_window() {
        let sliding_window = SlidingWindow::new(10, Duration::from_millis(100));
        let counter =  Arc::new(AtomicUsize::new(0));
        let reqs_diff = Arc::new(Mutex::new((Vec::new(), Instant::now())));

        (0..4).for_each(|_| {
            let sw = sliding_window.clone();
            let c = counter.clone();
            let rd = reqs_diff.clone();
            thread::spawn(move || {
                loop {
                    if sw.acquire() {
                        c.fetch_add(1, Ordering::SeqCst);
                        let (ref mut vec, ref mut time) = *rd.lock().unwrap();
                        let duration = Instant::now().saturating_duration_since(*time).as_millis();
                        *time = Instant::now();
                        vec.push(duration);
                    }
                }
            });
        });

        let start = Instant::now();
        while counter.load(Ordering::SeqCst) < 500 {
            spin_loop();
        }

        // 10 reqs refilled each 100 ms
        // to get 500 reqs, we have to spend 500/10 = 50, 50 * 100ms = 5000ms
        assert!(start.elapsed().as_millis() >= 4800 && start.elapsed().as_millis() <= 5200);
        // average difference between consecutive requests
        let avg = reqs_diff.lock().unwrap().0.iter().map(|x| x.clone() as f32).reduce(|x, y| (x + y) / 2.0).unwrap();
        assert!(avg > 9.0 && avg < 11.0);
    }
}
