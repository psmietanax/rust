use std::thread;
use std::collections::HashMap;
use std::thread::{ThreadId, JoinHandle};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct ThreadLocal<T: Send + Copy> {
    map: Arc<Mutex<HashMap<ThreadId, T>>>
}

impl <T: Send + Copy> ThreadLocal<T> {
    pub fn new() -> Self {
        ThreadLocal {
            map: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn set(&self, elem: T) {
        self.map.lock().unwrap().insert(thread::current().id(), elem);
    }

    pub fn get(&self) -> Option<T> {
        self.map.lock().unwrap().get(&thread::current().id()).map(|&val| val)
    }
}

impl <T: Send + Copy> Clone for ThreadLocal<T> {
    fn clone(&self) -> Self {
        ThreadLocal {
            map: Arc::clone(&self.map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let thread_local = ThreadLocal::new();
        let result = Arc::new(Mutex::new(Vec::with_capacity(10)));

        (0..10).map(|idx| {
            let tl = thread_local.clone();
            let res = Arc::clone(&result);

            thread::spawn(move || {
                tl.set(idx);
                // allow each thread to write its own data
                thread::sleep(Duration::from_millis(10));
                res.lock().unwrap().push(tl.get());
            })
        }).collect::<Vec<JoinHandle<()>>>()
            .into_iter()
            .for_each(|jh| {
                jh.join();
            });

        let mut result_vec = &mut *result.lock().unwrap();
        result_vec.sort();

        assert_eq!(result_vec, &vec![Some(0), Some(1), Some(2), Some(3), Some(4), Some(5), Some(6), Some(7), Some(8), Some(9)]);
    }
}
