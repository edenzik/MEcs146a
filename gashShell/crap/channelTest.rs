use std::sync::mpsc::{channel};
use std::thread;

static NUMTHREADS :usize = 10;

fn main() {

    // Maybe consider a Vec of channels instead?
    // Initialize them all at once and pass in only the required
    // references to threads as they are spawned.

    let mut rx_stack= Vec::new();
    let mut tx_stack = Vec::new();
    let mut join_guards = Vec::new();

    // Initialize a Vec full of channels
    tx_stack.push(None);
    for _ in 0..(NUMTHREADS - 1) {
        let (tx, rx) = channel::<String>();
        rx_stack.push(Some(rx));
        tx_stack.push(Some(tx));
    }
    rx_stack.push(None);

    // Initialize each thread, connect together with channels
    for id in 0..NUMTHREADS {
        let thread_rx = rx_stack.pop().unwrap();
        let thread_tx = tx_stack.pop().unwrap();

        let thread = {

            // Thread either gets message from the pipe, or if this thread
            // is the first (no input channel), it sends a token down instead.
            let prev_msg = match thread_rx {
                Some(reciever)    => reciever.recv().ok().expect("Could not read from channel"),
                None            => format!("{}", id), 
            };
            println!("Thread {} carrying {}", id, prev_msg);
            // Thread takes the message from the previous match and sends it
            // into its pipe with the thread's id as well. If the thread is
            // last in line (no output channel), it prints the whole chain instead.
            match thread_tx {
                Some(sender)  => {sender.send(format!("{}",id)).ok().expect("Send failed");},
                None        => {println!("all done {}", prev_msg);}
            };

        };
        join_guards.push(thread::scoped(move || thread));


    }

}

