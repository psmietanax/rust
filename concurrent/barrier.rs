use std::sync::{Arc, Condvar, Mutex};

pub struct Barrier {
    data: Arc<(Mutex<u8>, Condvar)>
}

impl Barrier {
    pub fn new(threads: u8) -> Self {
        Barrier {
            data: Arc::new((Mutex::new(threads), Condvar::new()))
        }
    }

    pub fn wait(&self) {
        let mut threads = self.data.0.lock().unwrap();
        if *threads == 1 {
            self.data.1.notify_all();
        } else {
            *threads -= 1;
            while *threads > 1 {
                threads = self.data.1.wait(threads).unwrap();
            }
        }
    }
}

impl Clone for Barrier {
    fn clone(&self) -> Self {
        Barrier {
            data: Arc::clone(&self.data)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use super::*;

    #[test]
    fn test() {
        let barrier_in = Barrier::new(4);
        let barrier_out = Barrier::new(5);
        let result = Arc::new(Mutex::new(Vec::new()));
        for i in 1..5 {
            let b_in = barrier_in.clone();
            let b_out = barrier_out.clone();
            let res = Arc::clone(&result);
            thread::spawn(move || {
                b_in.wait();
                thread::sleep(Duration::from_millis(i * 10));
                res.lock().unwrap().push(i);
                b_out.wait();
            });
        }
        barrier_out.wait();
        let mut result = result.lock().unwrap();
        result.sort();
        assert_eq!(*result, vec![1, 2, 3, 4]);
    }
}
