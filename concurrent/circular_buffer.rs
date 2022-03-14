/*
 A thread-safe multi-consumer multi-producer queue.
 */
pub struct CircularBuffer<T> {
    mask: Arc<i32>,
    // data buffer
    buffer: Arc<UnsafeCell<Vec<Option<T>>>>,
    // keeps producer/consumer sequence to track progress on buffer
    seq: Arc<UnsafeCell<Vec<i32>>>,
    // cache-padded head sequence (producer)
    head_seq: Arc<CachePadded<AtomicI32>>,
    // cache-padded tail sequence (consumer)
    tail_seq: Arc<CachePadded<AtomicI32>>
}

unsafe impl <T> Send for CircularBuffer<T> { }

impl <T> Clone for CircularBuffer<T> {
    fn clone(&self) -> Self {
        CircularBuffer {
            mask: Arc::clone(&self.mask),
            buffer: Arc::clone(&self.buffer),
            seq: Arc::clone(&self.seq),
            head_seq: Arc::clone(&self.head_seq),
            tail_seq: Arc::clone(&self.tail_seq),
        }
    }
}

impl <T> CircularBuffer<T> where T: Debug {
    pub fn new(mut capacity: usize) -> Self {
        let mask = capacity.next_power_of_two() as i32 - 1;
        capacity = mask as usize + 1;
        // pre-defined data buffer
        // buffer slots are defined as Option so both empty/non-empty values can be stored
        let mut buf: Vec<Option<T>> = Vec::with_capacity(capacity);
        buf.resize_with(capacity, || None);
        // pre-defined sequence buffer
        let seq: Vec<i32> = (0..capacity as i32).collect();

        CircularBuffer {
            mask: Arc::new(mask),
            buffer: Arc::new(UnsafeCell::new(buf)),
            seq: Arc::new(UnsafeCell::new(seq)),
            head_seq: Arc::new(CachePadded::new(AtomicI32::new(0))),
            tail_seq: Arc::new(CachePadded::new(AtomicI32::new(0)))
        }
    }

    pub fn offer(&self, elem: T) {
        let mask = *self.mask;
        let capacity = mask + 1;
        let mut tail_seq = i32::MIN;
        let mut head_seq;
        let mut seq_offset;
        loop {
            head_seq = self.head_seq.load(Ordering::Acquire);
            seq_offset = (head_seq & mask) as usize;
            let mut seq = unsafe {
                (&mut *self.seq.get())[seq_offset]
            };
            if seq < head_seq {
                if head_seq - capacity  >= tail_seq {
                    tail_seq = self.tail_seq.load(Ordering::Acquire);
                    if head_seq - capacity >= tail_seq {
                        spin_loop();
                        continue;
                    }
                }
                seq = head_seq + 1;
            }
            if seq <= head_seq &&
                self.head_seq.compare_exchange_weak(head_seq, head_seq + 1, Ordering::Release, Ordering::Relaxed).is_ok() {
                break;
            }
        }

        unsafe {
            let buf = &mut *self.buffer.get();
            let idx = (head_seq & mask) as usize;
            buf[idx] = Some(elem);

            let seq = &mut *self.seq.get();
            seq[seq_offset] = head_seq + 1;
        }
    }

    pub fn poll(&self) -> T {
        let mask = *self.mask;
        let capacity = mask + 1;

        let mut head_seq = -1;
        let mut tail_seq;
        let mut seq_offset;
        loop {
            tail_seq = self.tail_seq.load(Ordering::Acquire);
            seq_offset = (tail_seq & mask) as usize;
            let mut seq = unsafe {
                (&mut *self.seq.get())[seq_offset]
            };

            let expected_seq = tail_seq + 1;
            if seq < expected_seq {
                if tail_seq >= head_seq {
                    head_seq = self.head_seq.load(Ordering::Acquire);
                    if tail_seq == head_seq {
                        spin_loop();
                        continue;
                    }
                }
                seq = expected_seq + 1;
            }
            if seq <= expected_seq &&
                self.tail_seq.compare_exchange_weak(tail_seq, tail_seq + 1, Ordering::Release, Ordering::Relaxed).is_ok() {
                break;
            }
        }
        unsafe {
            let buf = &mut *self.buffer.get();
            let idx = (tail_seq & mask) as usize;

            let ret = buf[idx].take().unwrap()
                ;

            let seq = &mut *self.seq.get();
            seq[seq_offset] = tail_seq + capacity;

            ret
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use std::thread;

    #[test]
    fn test() {
        let queue = CircularBuffer::new(100);

        for i in 0..4 {
            let q = queue.clone();
            thread::spawn(move || {
                for j in 0..1_000 {
                    q.offer(i * 1_000 + j);
                }
            });
        }

        let mut counter = 0;
        let mut sum = 0;
        while counter < 4 * 1_000 {
            sum += queue.poll();
            counter += 1;
        }

        assert_eq!(sum, 7998000);
    }

    #[test]
    fn test_perf() {
        let mut timings = Vec::with_capacity(10);

        for _ in 1..10 {
            let queue = CircularBuffer::new(100_000);
            let q2 = queue.clone();

            let start = Instant::now();
            thread::spawn(move || {
                for i in 0..1_000_000 {
                    q2.offer(i);
                }
            });

            let mut counter = 0;
            let mut sum: i128 = 0;
            while counter < 1_000_000 {
                sum += queue.poll();
                counter += 1;
            }

            timings.push(start.elapsed().as_millis())
        }
        timings.sort();

        println!("Timings ordered ascending: {:?}", timings);
    }
}
