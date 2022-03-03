use std::collections::BinaryHeap;
use std::hash::{Hash, Hasher};
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime};
use chrono::Local;

use crate::delayed_queue::{DelayedQueue, ScheduledEntry};

type Task = Box<dyn FnOnce() + Send>;

struct TaskScheduler {
    task_queue: DelayedQueue<Task>,
    workers: Arc<Vec<JoinHandle<()>>>,
    is_active: Arc<AtomicBool>
}

impl Clone for TaskScheduler {
    fn clone(&self) -> Self {
        TaskScheduler {
            task_queue: self.task_queue.clone(),
            workers: Arc::clone(&self.workers),
            is_active: Arc::clone(&self.is_active)
        }
    }
}

impl TaskScheduler {
    pub fn new(capacity: usize) -> Self {
        let task_queue: DelayedQueue<Task> = DelayedQueue::new();
        let is_active = Arc::new(AtomicBool::new(true));
        let workers: Vec<JoinHandle<()>> = (0..capacity)
            .map(|_| (task_queue.clone(), is_active.clone()))
            .map(|(bq, ia)| {
                thread::spawn(move || {
                    while ia.load(Ordering::Acquire) {
                        if let Some(task) = bq.get() {
                            (task.entry())();
                        } else {
                            break;
                        }
                    }
                })
            }).collect();

        TaskScheduler {
            task_queue,
            workers: Arc::new(workers),
            is_active
        }
    }
    pub fn schedule<F>(&self, task: F, delay: Duration)
        where F: FnOnce() + Send + 'static {
        self.task_queue.offer(ScheduledEntry::of(Box::new(task), SystemTime::now().add(delay)));
    }
    pub fn shutdown(&self) {
        if self.is_active.compare_exchange(true, false, Ordering::Release, Ordering::Relaxed).unwrap() {
            self.task_queue.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::time::UNIX_EPOCH;
    use chrono::Local;
    use super::*;

    #[test]
    fn test_schedule_ascending_order() {
        let data = Arc::new(Mutex::new(Vec::new()));
        let data2 = Arc::clone(&data);
        let data3 = Arc::clone(&data);
        let data4 = Arc::clone(&data);

        let ts = TaskScheduler::new(1);

        ts.schedule(move || { data2.lock().unwrap().push(1) }, Duration::from_millis(10));
        ts.schedule(move || { data3.lock().unwrap().push(2) }, Duration::from_millis(20));
        ts.schedule(move || { data4.lock().unwrap().push(3) }, Duration::from_millis(30));

        thread::sleep(Duration::from_millis(35));

        assert_eq!(*data.lock().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_schedule_descending_order() {
        let data = Arc::new(Mutex::new(Vec::new()));
        let data2 = Arc::clone(&data);
        let data3 = Arc::clone(&data);
        let data4 = Arc::clone(&data);

        let ts = TaskScheduler::new(1);

        ts.schedule(move || { data2.lock().unwrap().push(1) }, Duration::from_millis(30));
        ts.schedule(move || { data3.lock().unwrap().push(2) }, Duration::from_millis(20));
        ts.schedule(move || { data4.lock().unwrap().push(3) }, Duration::from_millis(10));

        thread::sleep(Duration::from_millis(35));

        assert_eq!(*data.lock().unwrap(), vec![3, 2, 1]);
    }

    #[test]
    fn test_schedule_one_worker() {
        let data = Arc::new(Mutex::new(Vec::new()));
        let data2 = Arc::clone(&data);
        let data3 = Arc::clone(&data);

        let ts = TaskScheduler::new(1);

        ts.schedule(move || {
            thread::sleep(Duration::from_millis(10));
            data2.lock().unwrap().push(1)
        }, Duration::from_millis(10));
        thread::sleep(Duration::from_millis(5));
        ts.schedule(move || {
            thread::sleep(Duration::from_millis(10));
            data3.lock().unwrap().push(2)
        }, Duration::from_millis(10));

        thread::sleep(Duration::from_millis(20));
        //                         * task2 ready [worker1]
        //                 * task2 sleep [worker1]
        //                 * task1 ready [worker1]
        //         * task1 sleep [worker1]
        //    * schedule task2
        // * schedule task1
        // |-10ms-||-10ms-||-10ms-||-10ms-|
        //                     * sleep and validate [main thread]
        assert_eq!(*data.lock().unwrap(), vec![1]);
    }

    #[test]
    fn test_schedule_two_workers() {
        let data = Arc::new(Mutex::new(Vec::new()));
        let data2 = Arc::clone(&data);
        let data3 = Arc::clone(&data);

        let ts = TaskScheduler::new(2);

        ts.schedule(move || {
            thread::sleep(Duration::from_millis(10));
            data2.lock().unwrap().push(1)
        }, Duration::from_millis(10));
        thread::sleep(Duration::from_millis(5));
        ts.schedule(move || {
            thread::sleep(Duration::from_millis(10));
            data3.lock().unwrap().push(2)
        }, Duration::from_millis(10));

        thread::sleep(Duration::from_millis(25));
        //                    * task2 ready [worker2]
        //                 * task1 ready [worker1]
        //            * task2 sleep [worker2]
        //         * ask1 sleep [worker1]
        //    * schedule task2
        // * schedule task1
        // |-10ms-||-10ms-||-10ms-||-10ms-|
        //                     * sleep and validate [main thread]
        assert_eq!(*data.lock().unwrap(), vec![1, 2]);
    }
}
