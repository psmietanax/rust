use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{Receiver, Sender, channel};

fn broadcaster<T: Clone + Send + 'static>() -> (Sender<T>, Broadcaster<T>) {
    let (sender, receiver) = channel::<T>();
    let broadcaster = Broadcaster { senders: Arc::new(Mutex::new(Vec::new())) };

    let br = broadcaster.clone();
    thread::spawn(move || {
        loop {
            match receiver.recv() {
                Ok(val) => {
                    for sender in br.senders.lock().unwrap().iter() {
                        sender.send(val.clone());
                    }
                },
                Err(_) => {
                    break;
                }
            }
        }
    });

    (sender, broadcaster)
}

struct Broadcaster<T: Clone + Send> {
    senders: Arc<Mutex<Vec<Sender<T>>>>
}

impl <T: Clone + Send> Clone for Broadcaster<T> {
    fn clone(&self) -> Self {
        Broadcaster { senders: Arc::clone(&self.senders) }
    }
}

impl <T: Clone + Send> Broadcaster<T> {
    fn subscribe(&self) -> Receiver<T> {
        let (sender, receiver) = channel();
        self.senders.lock().unwrap().push(sender);
        receiver
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Barrier;

    #[test]
    fn test() {
        let (sd, br) = broadcaster();
        let br1 = br.clone();
        let br2 = br.clone();
        let barrier = Arc::new(Barrier::new(3));

        let bar1 = Arc::clone(&barrier);
        let t1 = thread::spawn(move || {
            let rc = br1.subscribe();
            bar1.wait();
            let mut result = Vec::new();
            loop {
                let val = rc.recv().unwrap();
                if val < 0 {
                    break;
                }
                result.push(val);
            }
            result
        });

        let bar2 = Arc::clone(&barrier);
        let t2 = thread::spawn(move || {
            let rc = br2.subscribe();
            bar2.wait();
            let mut result = Vec::new();
            loop {
                let val = rc.recv().unwrap();
                if val < 0 {
                    break;
                }
                result.push(val);
            }
            result
        });

        barrier.wait();
        for i in 1..10 {
            sd.send(i);
        }
        sd.send(-1);

        let vec1 = t1.join().unwrap();
        let vec2 = t2.join().unwrap();

        assert_eq!(vec1, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(vec2, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
