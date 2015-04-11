#![feature(process)]
#![feature(io)]

use std::process::{self, Command, Stdio};
use std::str;
use std::io::Read;

const COMMAND :&'static str = "date";

fn main() {
    let args: &[_] = &["-c", COMMAND];
    let cmd = Command::new("./gash").args(args).stdout(Stdio::capture()).spawn().unwrap();
    let iter = StdOutIter{ out : cmd.stdout.unwrap() };
    for text in iter {
        println!("{}", text);
    }
}

// Struct to encapsulate iteration over stdout from a spawned process
// Calling next reads the next buffer length, chops it to size,
// or returns None when the pipe is done.
const BUFFER_SIZE :usize = 80;
struct StdOutIter {
    out: process::ChildStdout,
}
impl<'a> Iterator for StdOutIter {
    type Item = String;

    fn next(& mut self) -> Option<String> {
        let mut buffer_array : [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let buffer = &mut buffer_array;

        let output_str = match self.out.read(buffer) {
            Ok(length) => if length == 0 { return None }
            else { str::from_utf8(&buffer[0..length]) },
            Err(_)   => { return None },
        };

        match output_str {
            Ok(string) => Some(string.to_string()),
            Err(_) => { println!("Error: Output not UTF8 encoding. Read failed.");
                None }
        }

    }
}