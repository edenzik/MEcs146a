use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::mpsc;
use std::thread;

static NUMTHREADS :usize = 3;

struct BufferStruct<'a> {
    some_text: &'a str,
    length: usize
}

fn main<'a>() {

    // Maybe consider a Vec of channels instead?
    // Initialize them all at once and pass in only the required
    // references to threads as they are spawned.

    let mut channels = Vec::new();

    // Initialize a Vec full of channels
    for _ in 0..(NUMTHREADS - 1) {
        channels.push(channel::<BufferStruct>());
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

        thread::scoped(move || {

            // Thread either gets message from the pipe, or if this thread
            // is the first (no input channel), it sends a token down instead.
            let prev_msg = match thread_rx {
                Some(sender)    => sender.recv().ok()
                                    .expect("Could not read from channel"),
                None            => BufferStruct{ some_text: "", length: 0 },
            };

            // Thread takes the message from the previous match and sends it
            // into its pipe with the thread's id as well. If the thread is
            // last in line (no output channel), it prints the whole chain instead.
            match thread_tx {
                Some(receiver)  => {receiver.send(BufferStruct{
                        some_text: format!("Thread {} {}", id, prev_msg.some_text)
                        .as_slice(),
                        length: (prev_msg.length + 9)});}
                None            => {println!("{} \n all done!", prev_msg.some_text);}
            };

        });

    }

}