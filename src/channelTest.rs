use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::mpsc;
use std::thread;

static NUMTHREADS :usize = 3;

fn main() {

    // Maybe consider a Vec of channels instead?
    // Initialize them all at once and pass in only the required
    // references to threads as they are spawned.

    let mut channels = Vec::new();

    // Initialize a Vec full of channels
    for _ in 0..(NUMTHREADS - 1) {
        channels.push(channel::<String>());
    }

    // Initialize each thread, connect together with channels
    for id in 0..NUMTHREADS {

        let thread_rx = match id {
            0               => None,
            n               => Some(&channels[n-1].1),
        };
        let thread_tx = match id {
            x if x == NUMTHREADS-1  => None,
            _                       => Some(&channels[id].0),
        };

        thread::spawn(move || {

            let prev_msg = match thread_rx {
                Some(sender)    => sender.recv().ok()
                                    .expect("Could not read from channel").as_slice(),
                None            => "**",
            };

            match thread_tx {
                Some(receiver)  => {receiver.send(
                    format!("Thread {} {}", id, prev_msg));}
                None            => {println!("{} \n all done!", prev_msg);}
            };

        });

    }

}