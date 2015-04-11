#![feature(process)]
#![feature(io)]
#![feature(old_io)]


use std::process::{self, Command, Stdio};
use std::str;
use std::io::Read;
use std::old_io;

// const COMMAND :&'static str = "date";

fn main() {
    let mut stdin = old_io::BufferedReader::new(old_io::stdin());
    loop {
        old_io::stdio::print("Type requested gash command: ");
        old_io::stdio::flush();

        // Try to read from stdin
        // If successful, create a GashCommandLine, otherwise let user try again
        let command_string = match stdin.read_line() {
            Ok(input_line) => input_line,
            Err(msg) => { println!("Failed to read from stdin: {}", msg); continue; }
        };


        // Replace &command_string here with COMMAND to send static "date" command
        let args: &[_] = &["-c", &command_string];
        let cmd = Command::new("./gash").args(args).stdout(Stdio::capture()).spawn().unwrap();
        let iter = StdOutIter{ out : cmd.stdout.unwrap() };
        for text in iter {
            print!("{}", text);
        }
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