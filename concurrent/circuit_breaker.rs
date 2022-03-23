use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use crate::circuit_breaker::State::{Closed, HalfOpen, Open};

enum State {
    Open,
    HalfOpen,
    Closed
}

#[derive(Debug)]
enum Error<E> {
    Custom(E),
    Rejected
}

trait Windowing {
    fn register_call(&mut self, was_success_call: bool);

    fn should_open(&self) -> bool;

    fn reset(&mut self);
}


// Count-based sliding window
struct WindowingCount {
    buffer: Vec<bool>,
    counter: usize,
    threshold: f32
}

impl WindowingCount {
    fn new(count: usize, threshold: f32) -> Self {
        WindowingCount {
            buffer: vec![true; count],
            counter: 0,
            threshold
        }
    }
}

impl Windowing for WindowingCount {
    fn register_call(&mut self, was_success_call: bool) {
        self.buffer[self.counter] = was_success_call;
        self.counter = (self.counter + 1) % self.buffer.len();
    }
    fn should_open(&self) -> bool {
        let failure_count = self.buffer.iter().filter(|&&was_success_call| !was_success_call).count();
        if failure_count as f32 / self.buffer.len() as f32 >= self.threshold {
            true
        } else {
            false
        }
    }
    fn reset(&mut self) {
        self.buffer.fill(true);
    }
}

// Time-based sliding window
struct WindowingTime {
    // (seconds since EPOCH, success count, failure count)
    buffer: Vec<(u64, u64, u64)>,
    threshold: f32
}

impl WindowingTime {
    fn new(seconds: usize, threshold: f32) -> Self {
        let since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        WindowingTime {
            buffer: vec![(since_epoch, 0, 0); seconds],
            threshold
        }
    }
}

impl Windowing for WindowingTime {
    fn register_call(&mut self, was_success_call: bool) {
        let since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let idx = (since_epoch % self.buffer.len() as u64) as usize;
        let (last_since_epoch, success_count, failure_count) = self.buffer[idx];
        if last_since_epoch < since_epoch {
            if was_success_call {
                self.buffer[idx] = (since_epoch, 1, 0);
            } else {
                self.buffer[idx] = (since_epoch, 0, 1);
            }
        } else {
            if was_success_call {
                self.buffer[idx] = (since_epoch, success_count + 1, failure_count);
            } else {
                self.buffer[idx] = (since_epoch, success_count, failure_count + 1);
            }
        }
    }
    fn should_open(&self) -> bool {
        let since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let (success_count, failure_count) = self.buffer.iter()
            .map(|&(last_since_epoch, success_count, failure_count)| {
                if (since_epoch - last_since_epoch) as usize <= self.buffer.len() {
                    (success_count, failure_count)
                } else {
                    (0, 0)
                }
            }).fold((0 as u64, 0 as u64), |(sc1, fc1), (sc2, fc2)| (sc1 + sc2, fc1 + fc2));
        if failure_count + success_count > 0 && failure_count as f32 / (success_count + failure_count) as f32 >= self.threshold {
            true
        } else {
            false
        }
    }
    fn reset(&mut self) {
        let since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        self.buffer.fill((since_epoch, 0, 0));
    }
}

struct SharedData {
    state: State,
    windowing: Box<dyn Windowing>,
    duration_in_open_state: Duration,
    state_changed: Instant
}

unsafe impl Send for SharedData { }

pub struct CircuitBreaker {
    data: Arc<Mutex<SharedData>>
}

impl Clone for CircuitBreaker {
    fn clone(&self) -> Self {
        CircuitBreaker {
            data: Arc::clone(&self.data)
        }
    }
}

impl CircuitBreaker {
    pub fn new_counted(count: usize, threshold: f32, duration_in_open_state: Duration) -> Self {
        assert!(count > 0, "Count must be greater than zero");
        assert!(threshold > 0.0 && threshold < 1.0, "Threshold must be within the range (0; 1)");

        CircuitBreaker {
            data: Arc::new(
                Mutex::new(
                    SharedData {
                        state: State::Closed,
                        windowing: Box::new(WindowingCount::new(count, threshold)),
                        duration_in_open_state,
                        state_changed: Instant::now()
                    }
                )
            )
        }
    }

    pub fn new_timed(seconds: usize, threshold: f32, duration_in_open_state: Duration) -> Self {
        assert!(seconds > 0, "Seconds must be greater than zero");
        assert!(threshold > 0.0 && threshold < 1.0, "Threshold must be within the range (0; 1)");

        CircuitBreaker {
            data: Arc::new(
                Mutex::new(
                    SharedData {
                        state: State::Closed,
                        windowing: Box::new(WindowingTime::new(seconds, threshold)),
                        duration_in_open_state,
                        state_changed: Instant::now()
                    }
                )
            )
        }
    }

    pub fn execute<F, R, E>(&self, f: F) -> Result<R, Error<E>>
    where F: FnOnce() -> Result<R, E> {
        let mut data = self.data.lock().unwrap();
        loop {
            match data.state {
                State::Closed => {
                    match f() {
                        Ok(v) => {
                            data.windowing.register_call(true);
                            return Ok(v);
                        },
                        Err(e) => {
                            data.windowing.register_call(false);
                            if data.windowing.should_open() {
                                data.state = Open;
                                data.state_changed = Instant::now();
                            }
                            return Err(Error::Custom(e));
                        }
                    }
                }
                State::Open => {
                    if data.state_changed.elapsed() > data.duration_in_open_state {
                        data.state = HalfOpen;
                        data.state_changed = Instant::now();
                    } else {
                        return Err(Error::Rejected);
                    }
                },
                State::HalfOpen => {
                    match f() {
                        Ok(v) => {
                            data.windowing.reset();
                            data.state = Closed;
                            data.state_changed = Instant::now();
                            return Ok(v);
                        },
                        Err(e) => {
                            data.state = Open;
                            data.state_changed = Instant::now();
                            return Err(Error::Custom(e));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::UNIX_EPOCH;
    use super::*;

    #[test]
    fn test_windowingCount() {
        // CB should be open if 3 out of 5 last calls failed
        let mut windowing = WindowingCount::new(5, 0.6);
        windowing.register_call(true);
        windowing.register_call(false);
        windowing.register_call(true);
        windowing.register_call(false);
        windowing.register_call(true);
        // 2/5 failed
        assert_eq!(windowing.should_open(), false);
        // 3/5 failed
        windowing.register_call(false);
        assert_eq!(windowing.should_open(), true);
        // reset should clear out all failures
        windowing.reset();
        windowing.register_call(false);
        // 1/5 failed
        assert_eq!(windowing.should_open(), false);
    }

    #[test]
    fn test_windowingTime() {
        // wait for the beginning of the next second
        let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH);
        let ms = (since_the_epoch.unwrap().as_millis() % 1000) as u64;
        thread::sleep(Duration::from_millis(1000 - ms));

        // CB should be open if 60% of calls within last 2 seconds failed
        let mut windowing = WindowingTime::new(2, 0.6);
        windowing.register_call(true);
        thread::sleep(Duration::from_millis(500));
        windowing.register_call(false);
        thread::sleep(Duration::from_millis(500));
        windowing.register_call(true);
        thread::sleep(Duration::from_millis(500));
        windowing.register_call(false);
        thread::sleep(Duration::from_millis(500));
        windowing.register_call(true);
        // 33% failed
        assert_eq!(windowing.should_open(), false);

        thread::sleep(Duration::from_millis(500));
        windowing.register_call(false);
        // 50% failed
        assert_eq!(windowing.should_open(), false);
        windowing.register_call(false);
        // 60% failed
        assert_eq!(windowing.should_open(), true);

        // reset should clear out all failures
        windowing.reset();
        windowing.register_call(true);
        windowing.register_call(false);
        // 50% failed
        assert_eq!(windowing.should_open(), false);
    }

    #[test]
    fn test_circuitBreaker_windowingCount() {
        let cb = CircuitBreaker::new_counted(10, 0.6, Duration::from_secs(2));
        let cb2 = cb.clone();
        let cb3 = cb.clone();
        let result = Arc::new(Mutex::new(Vec::new()));
        let result2 = Arc::clone(&result);

        let success_worker = thread::spawn(move || {
            for i in 0..10 {
                let res = Arc::clone(&result2);
                cb2.execute(move || {
                    res.lock().unwrap().push(i);
                    if 1 > 0 {
                        Ok(i)
                    } else {
                        Err("oops")
                    }
                });
                thread::sleep(Duration::from_millis(200));
            }
        });

        let failure_worker = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            for i in 0..10 {
                cb3.execute(move || {
                    if 1 < 0 {
                        Ok(i)
                    } else {
                        Err("oops")
                    }
                });
                thread::sleep(Duration::from_millis(100));
            }
        });

        success_worker.join();
        failure_worker.join();

        assert_eq!(result.lock().unwrap().len() < 10, true);
    }

    #[test]
    fn test_circuitBreaker_windowingTime() {
        let cb = CircuitBreaker::new_timed(2, 0.6, Duration::from_secs(2));
        let cb2 = cb.clone();
        let cb3 = cb.clone();
        let result = Arc::new(Mutex::new(Vec::new()));
        let result2 = Arc::clone(&result);

        let success_worker = thread::spawn(move || {
            for i in 0..10 {
                let res = Arc::clone(&result2);
                cb2.execute(move || {
                    res.lock().unwrap().push(i);
                    if 1 > 0 {
                        Ok(i)
                    } else {
                        Err("oops")
                    }
                });
                thread::sleep(Duration::from_millis(200));
            }
        });

        let failure_worker = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            for i in 0..10 {
                cb3.execute(move || {
                    if 1 < 0 {
                        Ok(i)
                    } else {
                        Err("oops")
                    }
                });
                thread::sleep(Duration::from_millis(100));
            }
        });

        success_worker.join();
        failure_worker.join();

        assert_eq!(result.lock().unwrap().len() < 10, true);
    }
}
