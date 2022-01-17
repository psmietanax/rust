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
        while let Some(num) = receiver_in.recv().unwrap() {
            println!("[THREAD2]: {}", num_to_string(num));
            if num == n {
                sender_out.send(None);
            } else {
                sender_out.send(Some(num + 1));
            }
        }
        sender_out.send(None);
    });

    sender_in.send(Some(1));

    while let Some(num) = receiver_out.recv().unwrap() {
        println!("[THREAD1]: {}", num_to_string(num));
        if num == n {
            sender_in.send(None);
        } else {
            sender_in.send(Some(num + 1));
        }
    }
    sender_in.send(None);
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
