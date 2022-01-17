use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

fn main() {
    fizz_buzz(20);
}

fn fizz_buzz(n: usize) {
    if n == 0 {
        return;
    }

    let (sender_in, receiver_in) = mpsc::channel();
    let (sender_out, receiver_out) = mpsc::channel();

    thread::spawn(move || {
        print_fizz_buzz(2, n, sender_out, receiver_in);
    });

    sender_in.send(Some(1));

    print_fizz_buzz(1, n, sender_in, receiver_out);
}

fn print_fizz_buzz(thread_id: usize, n: usize, sender: Sender<Option<usize>>, receiver: Receiver<Option<usize>>) {
    while let Some(num) = receiver.recv().unwrap() {
        println!("[THREAD{}]: {}", thread_id, num_to_string(num));
        if num == n {
            sender.send(None);
        } else {
            sender.send(Some(num + 1));
        }
    }
    sender.send(None);
}

fn num_to_string(n: usize) -> String {
    if n % 15 == 0 {
        "fizzbuzz".to_string()
    } else if n % 3 == 0 {
        "fizz".to_string()
    } else if n % 5 == 0 {
        "buzz".to_string()
    } else {
        n.to_string()
    }
}
