use crossbeam::atomic::AtomicCell;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::thread;

fn main() {
    
}

enum State {
    Open,
    HalfOpen(Instant),
    Closed
}

trait Windowing {
    fn register_result(&self, was_successful: bool) -> (usize, usize);
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
    fn register_result(&self, was_successful: bool) -> (usize, usize) {
        let (ref mut slots, ref mut counter) = *self.slots.lock().unwrap();
        *counter = (*counter + 1) % slots.len();
        slots[*counter] = was_successful;
        (*counter, *counter)
    }
}

impl Clone for WindowingCount {
    fn clone(&self) -> Self {
        WindowingCount {
            slots: Arc::clone(&self.slots)
        }
    }
}

struct Data<W: Windowing> {
    windowing: Arc<W>,
    open_state_duration: Arc<Duration>
}

impl Data<WindowingCount> {
    fn new_windowing_counter(measurements: usize, open_state_duration: Duration) -> Self {
        Data {
            windowing: Arc::new(WindowingCount::new(measurements)),
            open_state_duration: Arc::new(open_state_duration)
        }
    }
}

impl <W: Windowing> Clone for Data<W> {
    fn clone(&self) -> Self {
        Data {
            windowing: Arc::clone(&self.windowing),
            open_state_duration: Arc::clone(&self.open_state_duration)
        }
    }
}

struct StateData {
    state: AtomicCell<State>
}

impl StateData {
    fn new() -> Self {
        StateData {
            state: AtomicCell::new(State::Closed)
        }
    }
}

// check status
// if closed/half-open:
//   execute
//   get result
//   update stats
//   change status if required
// if open:
//   reject

