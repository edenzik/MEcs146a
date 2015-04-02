extern crate getopts;


// use getopts::{optopt, getopts};
// use std::old_io::BufferedReader;
use std::process;
// use std::old_io::stdin;
// use std::{old_io, os};
use std::io::Read;
use std::str;
use std::thread;

struct StdOutIter {
    out: process::ChildStdout,
}

static test_string: &'static str = "This is \n a test \n string, yo.\n";

impl<'a> Iterator for StdOutIter {
    type Item = String;

    fn next(& mut self) -> Option<String> {
        let mut buffer_array : [u8; 80] = [0; 80];
        let buffer = &mut buffer_array;
        
        let output_str = match self.out.read(buffer) {
            Ok(length) => { if length == 0 { println!("found no chars"); return None }
                            else { str::from_utf8(&buffer[0..length]) } },
            Err(_)   => { return None },
        };

        match output_str {
            Ok(string) => Some(string.to_string()),
            Err(_) => panic!("failed to convert stdin to String"),
        }

    }
}

fn main() {


    let cmd = process::Command::new("wc")
        .stdin(process::Stdio::capture())
        .stdout(process::Stdio::capture())
        .stderr(process::Stdio::capture()).spawn().unwrap();

    let stdout = cmd.stdout.unwrap();

    

    let process_reader = StdOutIter{ out : stdout };

    for output in process_reader {
        print!("{}(another eighty)", output);
    }
}
