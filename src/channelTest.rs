use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::mpsc;
use std::thread;

struct Mine {
    value: i32
}

static NUMTHREADS :i32 = 3;

fn main() {

    let mut chnl: (Sender, Receiver);
    let mut counter :i32 = 0;
    let mut in_pipe = Option::<Sender>;

    for id in 0..NUMTHREADS {

        chnl = channel::<String>();
        let thread_rx = pipe;
        let thread_tx = chnl.0.clone();
        let for_next_round = chnl.0.clone();

        thread::spawn(move || {

            let prev_msg = thread_rx.recv();

            thread_tx.send(format!("Thread {} {}", id, prev_msg));

        });



        in_pipe = for_next_round;
    }

}