use std::sync::mpsc::{channel};
use std::thread;
use std::time::duration::Duration;
use std::old_io::timer;

// static NUMTHREADS :usize = 3;

fn main() {


    let the_channel = channel();
    let (tx, rx) = the_channel;
    // let (tx1, rx1) = channel();
    let tx_s = Some(tx);

        println!("I am main, hear me roar!");

        let thread1 = thread::scoped(move || {
            println!("commencing thread run 1");

            let thread_tx = tx_s.unwrap();
            let msg = "abba";

            println!("I am thread {} and I am sending message {}", 1, msg);
            for _ in 0..5 {
                match thread_tx.send(msg) {
                    Ok(_)    => {}
                    Err(_)     => {println!("I wasn't able to send the message.");}
                };    
            }

            println!("Thread 1 done sending all messages, closing channel");
            // drop(thread_tx);                        

        });

        let thread2 = thread::scoped(move || {
            let thread_rx = rx;
            // let thread_tx1 = tx1;

            println!("Thread 2 going to sleep");
            let interval = Duration::milliseconds(5000);
            timer::sleep(interval);
            println!("Thread 2 woke up, checking channel");

            loop {
            match thread_rx.recv() {
                Ok(msg) => { println!("I am thread {} and I have received message {}", 2, msg); }
                Err(err) => { println!("Got Error: {}, assuming channel closed", err); break; }
            }

            
            }
        });

        thread1.join();
        thread2.join();

        // println!("Got final message: {}", rx1.recv().unwrap());
}