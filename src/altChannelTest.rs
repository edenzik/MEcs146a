use std::sync::mpsc::{channel};
use std::thread;

// static NUMTHREADS :usize = 3;

fn main() {


    let the_channel = channel();
    let (tx, rx) = the_channel;
    let (tx1, rx1) = channel();

        println!("I am main, hear me roar!");

        let thread1 = thread::scoped(move || {
            println!("commencing thread run 1");

            let thread_tx = tx;
            let msg = "abba";

            println!("I am thread {} and I am sending message {}", 1, msg);
            
            match thread_tx.send(msg) {
                Ok(_)    => {}
                Err(_)     => {println!("I wasn't able to send the message.");}
            };            

        });

        let thread2 = thread::scoped(move || {
            let thread_rx = rx;
            let thread_tx1 = tx1;

            let msg = thread_rx.recv().ok().unwrap();
            thread_tx1.send(msg).unwrap();

            println!("I am thread {} and I have received message {}", 2, msg);
        });

        thread1.join();
        thread2.join();

        println!("Got final message: {}", rx1.recv().unwrap());
}