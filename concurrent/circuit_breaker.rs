use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use crossbeam::atomic::AtomicCell;

#[derive(Debug, PartialEq)]
pub enum Error<E> {
    Custom(E),
    Rejected
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum State {
    Open(Instant),
    HalfOpen,
    Closed
}

pub trait Windowing {
    fn register_result(&self, was_successful: bool) -> (u64, u64);
    fn reset(&self);
}

// Count-based sliding window implemented with a circular array of N slots
struct WindowingCount {
    // results slots: true (successful execution), false (unsuccessful execution)
    // counter iterating over the circular array
    slots: Arc<Mutex<(Vec<bool>, usize)>>
}

impl WindowingCount {
    fn new(measurements: usize) -> Self {
        WindowingCount {
            slots: Arc::new(Mutex::new((vec![true; measurements], 0)))
        }
    }
}

impl Windowing for WindowingCount {
    fn register_result(&self, was_successful: bool) -> (u64, u64) {
        let (ref mut slots, ref mut counter) = *self.slots.lock().unwrap();
        slots[*counter] = was_successful;
        *counter = (*counter + 1) % slots.len();

        let successCount = slots.iter().filter(|&&was_successful| was_successful).count();
        (successCount as u64, (slots.len() - successCount) as u64)
    }

    fn reset(&self) {
        let (ref mut slots, _) = *self.slots.lock().unwrap();
        slots.fill(true);
    }
}

impl Clone for WindowingCount {
    fn clone(&self) -> Self {
        WindowingCount {
            slots: Arc::clone(&self.slots)
        }
    }
}

struct WindowingTime {
    // (seconds since EPOCH, success count, failure count)
    slots: Arc<Mutex<Vec<(u64, u64, u64)>>>
}

impl WindowingTime {
    fn new(seconds: usize) -> Self {
        WindowingTime {
            slots: Arc::new(Mutex::new(vec![(0, 0, 0); seconds]))
        }
    }
}

impl Windowing for WindowingTime {
    fn register_result(&self, was_successful: bool) -> (u64, u64) {
        let slots = &mut *self.slots.lock().unwrap();
        let secs_since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let idx = (secs_since_epoch % slots.len() as u64) as usize;
        let (last_since_epoch, success_count, failure_count) = slots[idx];
        if last_since_epoch < secs_since_epoch {
            if was_successful {
                slots[idx] = (secs_since_epoch, 1, 0);
            } else {
                slots[idx] = (secs_since_epoch, 0, 1);
            }
        } else {
            if was_successful {
                slots[idx] = (secs_since_epoch, success_count + 1, failure_count);
            } else {
                slots[idx] = (secs_since_epoch, success_count, failure_count + 1);
            }
        }

        slots.iter().map(|&(last_since_epoch, success_count, failure_count)| {
            if (secs_since_epoch - last_since_epoch) as usize <= slots.len() {
                (success_count, failure_count)
            } else {
                (0, 0)
            }
        }).fold((0, 0), |(sc1, fc1), (sc2, fc2)| (sc1 + sc2, fc1 + fc2))
    }

    fn reset(&self) {
        let slots = &mut *self.slots.lock().unwrap();
        slots.fill((0, 0, 0));
    }
}

impl Clone for WindowingTime {
    fn clone(&self) -> Self {
        WindowingTime {
            slots: Arc::clone(&self.slots)
        }
    }
}

struct Data<W: Windowing> {
    state: Arc<AtomicCell<State>>,
    windowing: Arc<W>,
    open_state_duration: Arc<Duration>,
    threshold: f32
}

impl Data<WindowingCount> {
    fn new(measurements: usize, threshold: f32, open_state_duration: Duration) -> Self {
        Data {
            state: Arc::new(AtomicCell::new(State::Closed)),
            windowing: Arc::new(WindowingCount::new(measurements)),
            open_state_duration: Arc::new(open_state_duration),
            threshold
        }
    }
}

impl Data<WindowingTime> {
    fn new(seconds: usize, threshold: f32, open_state_duration: Duration) -> Self {
        Data {
            state: Arc::new(AtomicCell::new(State::Closed)),
            windowing: Arc::new(WindowingTime::new(seconds)),
            open_state_duration: Arc::new(open_state_duration),
            threshold
        }
    }
}

impl <W: Windowing> Clone for Data<W> {
    fn clone(&self) -> Self {
        Data {
            state: Arc::clone(&self.state),
            windowing: Arc::clone(&self.windowing),
            open_state_duration: Arc::clone(&self.open_state_duration),
            threshold: self.threshold.clone()
        }
    }
}

pub struct CircuitBreaker<W: Windowing> {
    data: Data<W>
}

impl <W: Windowing> Clone for CircuitBreaker<W> {
    fn clone(&self) -> Self {
        CircuitBreaker {
            data: self.data.clone()
        }
    }
}

impl CircuitBreaker<WindowingCount> {
    fn new(measurements: usize, threshold: f32, open_state_duration: Duration) -> Self {
        CircuitBreaker {
            data: Data::<WindowingCount>::new(measurements, threshold, open_state_duration)
        }
    }
}

impl CircuitBreaker<WindowingTime> {
    fn new(seconds: usize, threshold: f32, open_state_duration: Duration) -> Self {
        CircuitBreaker {
            data: Data::<WindowingTime>::new(seconds, threshold, open_state_duration)
        }
    }
}

impl <W: Windowing> CircuitBreaker<W> {
    pub fn execute<F, R, E>(&self, f: F) -> Result<R, Error<E>> where F: FnOnce() -> Result<R, E> {
        loop {
            let state = self.data.state.load();
            match state {
                State::Closed => {
                    let result = f().map_err(|e| Error::Custom(e));
                    let (success, failure) = self.data.windowing.register_result(result.is_ok());
                    let failure_ratio = failure as f32 / (success as f32 + failure as f32);
                    if failure_ratio >= self.data.threshold {
                        self.data.state.compare_exchange(State::Closed, State::Open(Instant::now()));
                    }
                    return result;
                },
                State::Open(changed) => {
                    if Instant::now().duration_since(changed) > *self.data.open_state_duration {
                        self.data.state.compare_exchange(State::Open(changed), State::HalfOpen);
                    } else {
                        return Err(Error::Rejected);
                    }
                },
                State::HalfOpen => {
                    let result = f().map_err(|e| Error::Custom(e));
                    if result.is_ok() {
                        if self.data.state.compare_exchange(State::HalfOpen, State::Closed).is_ok() {
                            self.data.windowing.reset();
                        }
                    } else {
                        self.data.state.compare_exchange(State::HalfOpen, State::Open(Instant::now()));
                    }
                    return result;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Barrier;
    use super::*;

    #[test]
    fn test_WindowingCount_reject() {
        let cb = CircuitBreaker::<WindowingCount>::new(10, 0.6, Duration::from_millis(10));

        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        // after 6 failures, next executions are rejected
        let result = cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err(), Error::Rejected);
    }

    #[test]
    fn test_WindowingTime_reject() {
        let cb = CircuitBreaker::<WindowingTime>::new(1, 0.6, Duration::from_millis(10));

        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        // after 6 failures, next executions are rejected
        let result = cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err(), Error::Rejected);
    }

    #[test]
    fn test_WindowingCount_reject_and_recover() {
        let cb = CircuitBreaker::<WindowingCount>::new(10, 0.6, Duration::from_millis(10));

        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });

        thread::sleep(Duration::from_millis(10));
        // recover after 10 milliseconds
        cb.execute(|| { if 1 > 0 { Ok(1) } else { Err("oops") } });

        let result = cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err(), Error::Custom("oops"));
    }

    #[test]
    fn test_WindowingTime_reject_and_recover() {
        let cb = CircuitBreaker::<WindowingTime>::new(10, 0.6, Duration::from_millis(10));

        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
        cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });

        thread::sleep(Duration::from_millis(10));
        // recover after 10 milliseconds
        cb.execute(|| { if 1 > 0 { Ok(1) } else { Err("oops") } });

        let result = cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err(), Error::Custom("oops"));
    }

    #[test]
    fn test_WindowingCount_multiple_threads_failure() {
        let cb = CircuitBreaker::<WindowingCount>::new(10, 0.6, Duration::from_millis(100));
        let cb2 = cb.clone();
        let cb3 = cb.clone();
        let barrierIn = Arc::new(Barrier::new(2));
        let barrierIn2 = Arc::clone(&barrierIn);
        let barrierOut = Arc::new(Barrier::new(2));
        let barrierOut2 = Arc::clone(&barrierOut);
        let result = Arc::new(Mutex::new(Vec::new()));
        let result2 = Arc::clone(&result);

        let success_thread = thread::spawn(move || {
            for i in 0..3 {
                result2.lock().unwrap().push(cb2.execute(|| { if 1 > 0 { Ok(i) } else { Err("oops") } }));
            }
            barrierIn.wait();
            barrierOut.wait();
            for i in 3..5 {
                result2.lock().unwrap().push(cb2.execute(|| { if 1 > 0 { Ok(i) } else { Err("oops") } }));
            }
        });

        let failure_thread = thread::spawn(move || {
            // wait for success_thread to execute some successful tasks
            barrierIn2.wait();
            for _ in 0..6 {
                cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
            }
            barrierOut2.wait();
        });

        success_thread.join();
        failure_thread.join();

        let result = &*result.lock().unwrap();
        assert_eq!(result, &vec![Ok(0), Ok(1), Ok(2), Err(Error::Rejected), Err(Error::Rejected)])
    }

    #[test]
    fn test_WindowingTime_multiple_threads_failure() {
        let cb = CircuitBreaker::<WindowingTime>::new(10, 0.6, Duration::from_millis(100));
        let cb2 = cb.clone();
        let cb3 = cb.clone();
        let barrierIn = Arc::new(Barrier::new(2));
        let barrierIn2 = Arc::clone(&barrierIn);
        let barrierOut = Arc::new(Barrier::new(2));
        let barrierOut2 = Arc::clone(&barrierOut);
        let result = Arc::new(Mutex::new(Vec::new()));
        let result2 = Arc::clone(&result);

        let success_thread = thread::spawn(move || {
            for i in 0..3 {
                result2.lock().unwrap().push(cb2.execute(|| { if 1 > 0 { Ok(i) } else { Err("oops") } }));
            }
            barrierIn.wait();
            barrierOut.wait();
            for i in 3..5 {
                result2.lock().unwrap().push(cb2.execute(|| { if 1 > 0 { Ok(i) } else { Err("oops") } }));
            }
        });

        let failure_thread = thread::spawn(move || {
            // wait for success_thread to execute some successful tasks
            barrierIn2.wait();
            for _ in 0..6 {
                cb.execute(|| { if 1 > 0 { Err("oops") } else { Ok(1) } });
            }
            barrierOut2.wait();
        });

        success_thread.join();
        failure_thread.join();

        let result = &*result.lock().unwrap();
        assert_eq!(result, &vec![Ok(0), Ok(1), Ok(2), Err(Error::Rejected), Err(Error::Rejected)])
    }

    #[test]
    fn test_WindowingCount_multiple_threads_success() {
        let cb = CircuitBreaker::<WindowingTime>::new(10, 0.6, Duration::from_millis(10));
        let cb2 = cb.clone();
        let cb3 = cb.clone();
        let result = Arc::new(Mutex::new(Vec::new()));
        let result2 = Arc::clone(&result);
        let result3 = Arc::clone(&result);

        let start = Instant::now();

        let t1 = thread::spawn(move || {
            for i in 0..50 {
                result2.lock().unwrap().push(cb2.execute(|| { if 1 > 0 { Ok(2 * i + 1) } else { Err("oops")} }));
                thread::sleep(Duration::from_millis(10));
            }
        });

        let t2 = thread::spawn(move || {
            for i in 0..50 {
                result3.lock().unwrap().push(cb3.execute(|| { if 1 > 0 { Ok(2 * i) } else { Err("oops")} }));
                thread::sleep(Duration::from_millis(10));
            }
        });

        t1.join();
        t2.join();

        assert_eq!(start.elapsed().as_millis() < 600, true);

        let result = &*result.lock().unwrap();
        assert_eq!(result.len(), 100);
    }

}
