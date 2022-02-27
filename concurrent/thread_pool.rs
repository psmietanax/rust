use std::thread;
use std::sync::mpsc::{Receiver, Sender, channel, RecvError};
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, Barrier};
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use fp_rust::sync::CountDownLatch;
use std::time::{Instant, Duration};

type Task = Box<dyn FnOnce() + Send>;

struct BlockingQueue<T> {
    sender: Arc<Mutex<Option<Sender<T>>>>,
    receiver: Arc<Mutex<Receiver<T>>>
}

impl<T> BlockingQueue<T> {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender: Arc::new(Mutex::new(Some(sender))),
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn push(&self, e: T) {
        self.sender.lock().unwrap().as_ref().unwrap().send(e);
    }

    pub fn pop(&self) -> Result<T, RecvError> {
        self.receiver.lock().unwrap().recv()
    }

    pub fn stop(&self) {
        self.sender.lock().unwrap().take();
    }
}

impl<T> Clone for BlockingQueue<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

struct ThreadPool {
    workers: Arc<Vec<JoinHandle<()>>>,
    task_queue: BlockingQueue<Task>,
    is_active: Arc<AtomicBool>
}

impl Clone for ThreadPool {
    fn clone(&self) -> Self {
        ThreadPool {
            workers: Arc::clone(&self.workers),
            task_queue: self.task_queue.clone(),
            is_active: Arc::clone(&self.is_active)
        }
    }
}

impl ThreadPool {
    fn new(capacity: usize) -> Self {
        let task_queue: BlockingQueue<Task> = BlockingQueue::new();
        let is_active = Arc::new(AtomicBool::new(true));
        let workers: Vec<JoinHandle<()>> = (0..capacity)
            .map(|_| (task_queue.clone(), is_active.clone()))
            .map(|(bq, ia)| {
                thread::spawn(move || {
                    while ia.load(Ordering::Acquire) {
                        if let Ok(task) = bq.pop() {
                            task();
                        } else {
                            break;
                        }
                    }
                })
            }).collect();
        ThreadPool {
            workers: Arc::new(workers),
            task_queue,
            is_active
        }
    }
    fn execute<F>(&self, task: F)
        where F: FnOnce() + Send + 'static {
        if self.is_active.load(Ordering::Acquire) {
            self.task_queue.push(Box::new(task));
        } else {
            panic!("Thread pool is shutdown")
        }
    }
    fn shutdown(&self) {
        if self.is_active.compare_exchange(true, false, Ordering::Release, Ordering::Relaxed).unwrap() {
            self.task_queue.stop();
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Instant, Duration};
    use fp_rust::sync::CountDownLatch;

    #[test]
    fn test_thread_pool() {
        const THREAD_POOL_SIZE: usize = 5;
        const TASKS_COUNT: u64 = 5;

        let thread_pool = ThreadPool::new(THREAD_POOL_SIZE);
        let latch = CountDownLatch::new(TASKS_COUNT);

        let start = Instant::now();

        for _ in 0..TASKS_COUNT {
            let latch = latch.clone();
            thread_pool.execute(move || {
                thread::sleep(Duration::from_millis(100));
                latch.countdown();
            });
        }

        latch.wait();
        // there are 5 tasks and exactly 5 workers, so we'll have to wait 100ms
        assert_eq!(start.elapsed().as_millis() >= 100, true);
    }

    #[test]
    fn test_thread_pool_task_queueing() {
        const THREAD_POOL_SIZE: usize = 5;
        const TASKS_COUNT: u64 = 10;

        let thread_pool = ThreadPool::new(THREAD_POOL_SIZE);
        let latch = CountDownLatch::new(TASKS_COUNT);

        let start = Instant::now();

        for _ in 0..TASKS_COUNT {
            let latch = latch.clone();
            thread_pool.execute(move || {
                thread::sleep(Duration::from_millis(100));
                latch.countdown();
            });
        }

        latch.wait();
        // there are 10 tasks and only 5 workers, so we'll have to wait 200ms
        assert_eq!(start.elapsed().as_millis() >= 200, true);
    }

    #[test]
    #[should_panic(expected = "Thread pool is shutdown")]
    fn test_thread_shutdown() {
        const THREAD_POOL_SIZE: usize = 1;

        let thread_pool = ThreadPool::new(THREAD_POOL_SIZE);

        thread_pool.shutdown();

        thread_pool.execute(move || { });
    }
}
